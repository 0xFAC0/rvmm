use core::slice;
use std::{ptr::null_mut, io::{Write, Read}, path::Path, fs::File};

use kvm_bindings::{kvm_userspace_memory_region, KVM_MEM_LOG_DIRTY_PAGES};
use kvm_ioctls::{VmFd, Kvm, VcpuFd};
#[allow(unused)]
use log::{debug, error, info, warn};

use crate::mem_inspection::DisASM;

use super::{Vm, Ram};

type Result<T> = std::result::Result<T, kvm_ioctls::Error>;

#[allow(dead_code)]
#[derive(Default)]
pub struct VmBuilder {
    slot: Option<u32>,
    vm_fd: Option<VmFd>,
    vcpu_fd: Option<VcpuFd>,
    mem_size: Option<usize>,
    load_addr: u64,
    guest_phys_addr: Option<usize>,
    tss_addr: Option<u64>,
    // devices_fd: Vec<DeviceFd>,
    code: Vec<u8>,
}

#[allow(unused)]
impl VmBuilder {
    pub fn slot(mut self, slot: u32) -> Self { self.slot = Some(slot); self }
    pub fn vm_fd(mut self, vm_fd: VmFd) -> Self { self.vm_fd = Some(vm_fd); self }
    pub fn vcpu_fd(mut self, vcpu_fd: VcpuFd) -> Self { self.vcpu_fd = Some(vcpu_fd); self }
    pub fn mem_size(mut self, mem_size: usize) -> Self { self.mem_size = Some(mem_size); self }
    pub fn guest_phys_addr(mut self, guest_phys_addr: usize) -> Self { self.guest_phys_addr = Some(guest_phys_addr); self }
    pub fn tss_address(mut self, offset: u64) -> Self { self.tss_addr = Some(offset); self }

    pub fn build(mut self) -> Result<Vm> {
        // TODO Errors
        let slot = self.slot.unwrap_or(0);
        let guest_phys_addr = self.guest_phys_addr.expect("Missing guest_phys_addr");

        let mem_size = match self.mem_size {
            Some(user_conf_mem_size) => user_conf_mem_size,
            None => match self.code.len() {
                // Try to align memory (i dont
                // remember if its 0x10 or 0x1000)
                x if x > 0 => (x * 4) + (x * 4 % 0x10),         
                _ => panic!("Cant create VM without code to run")
            }
        };
        debug!("RAM size: 0x{mem_size:x}");

        // Size must be aligned to 0x1000 I guess
        // TMP To fix, determine alignment
        self.load_addr = match self.kvm_allocate_region(0, None, guest_phys_addr as u64, mem_size as u64) {
            Ok(addr) => addr,
            Err(e) => {
                error!("KVM allocate userspace memory region failed {e}");
                return Err(e);
            }
        };
        self.write_img();

        // TMP, prepare device creation
        // The address seems to be commonly used
        // default 0xfffb_c000
        let vm_fd = self.vm_fd.as_mut().expect("Missing VM fd, unreachable code wtf");
        // vm_fd.set_identity_map_address(0).map_err(|e| {error!("Set identity map addresse failed {e}"); e})?;
        //  TSS used for taskswitching
        // default 0xffff_d000
        // vm_fd.set_tss_address(0).map_err(|e| {error!("Set TSS address failed {e}"); e})?;;

        // IRQ = Interrupt Request
        // if vm_fd.check_extension(Cap::Irqchip) {
        //     info!("IRQ Chip available");
        // } else {
        //     error!("IRQ Chip unavailable");
        //     let mut cap: kvm_enable_cap = Default::default();
        //     cap.cap = KVM_CAP_SPLIT_IRQCHIP;
        //     cap.args[0] = 24;
        //     vm_fd.enable_cap(&cap)?;
        //     panic!("IRQ Chip unavailable");
        // }
        // vm_fd.create_irq_chip().map_err(|e| {error!("Create IRQ chip failed {e}"); e})?;

        // // Programmable Interval Timer, hw timer, used for tasks such as generating regular
        // // interrupts for OS Scheduling and maintaining system time or smtg like that
        // let pit_config = kvm_bindings::bindings::kvm_pit_config { flags: 1, pad: [0u32; 15] };
        // vm_fd.create_pit2(pit_config).map_err(|e| {error!("Create PIT2 failed {e}"); e})?;

        // debug!("Setting registers");
        // let mut regs = vcpu_fd.get_regs().unwrap();
        // //let mut sregs = vcpu.get_sregs().unwrap();
        // // TOFIX
        // regs.rip = 0x000000;
        // regs.rsp = 0x000000;
        // vcpu_fd.set_regs(&regs).expect("Could not set registers");

        // TODO vCPU ID 
        let vcpu_fd = vm_fd.create_vcpu(0)?;
        // Might be asm_code dependant
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        {
            let mut vcpu_sregs = vcpu_fd.get_sregs().unwrap();
            vcpu_sregs.cs.base = 0;
            vcpu_sregs.cs.selector = 0;

            // Added
            // vcpu_sregs.cs.g = 1;
            // vcpu_sregs.ds.g =1;
            // vcpu_sregs.fs.g =1;
            // vcpu_sregs.gs.g =1;
            // vcpu_sregs.es.g =1;
            // vcpu_sregs.ss.g =1;
            // vcpu_sregs.cr0  |=1;

            vcpu_fd.set_sregs(&vcpu_sregs).unwrap();
            debug!("Special registers set:");
            debug!("{vcpu_sregs:?}");

            let mut vcpu_regs = vcpu_fd.get_regs().unwrap();
            vcpu_regs.rip = guest_phys_addr as u64;
            // vcpu_regs.rax = 2;
            // vcpu_regs.rbx = 3;
            vcpu_regs.rflags = 2;

            vcpu_fd.set_regs(&vcpu_regs).unwrap();
        }

        debug!("vCPU added");
        debug!("{vcpu_fd:?}");
        self.vcpu_fd = Some(vcpu_fd);

        if self.code.len() == 0 {
            error!("No code loaded, can't run the VM without code. I decided to not build it");
            panic!("Attempt to build VM without code");
        }

        debug!("{:x?}", self.vcpu_fd);
        Ok( Vm {
            slot,
            vm_fd: self.vm_fd.unwrap(), 
            vcpu_fd: self.vcpu_fd.unwrap(),
            ram: Ram::new(self.load_addr, mem_size, guest_phys_addr as u64, self.tss_addr),
            devices_fd: vec![],
        })
    }

    pub fn load_asm(mut self, asm_code: &'static [u8]) -> Self {
        if asm_code.len() == 0 {
            panic!("can't load null asm code");
        }
        self.code = Vec::from(asm_code);
        self
    }

    /// TODO fix inconsistent type for load_addr between *mut u8 and u64
    fn write_asm(&mut self) {
        debug!("Writting code at RIP");
        unsafe {
            let mut vm_mem = slice::from_raw_parts_mut(self.load_addr as *mut u8, self.code.len());
            let written = vm_mem.write(self.code.as_ref()).expect("Could not write code to memory region");
            let vm_mem = slice::from_raw_parts(self.load_addr as *mut u8, self.code.len());
            vm_mem.disasm_all(self.load_addr as u64);
            debug!("{written}B written");
        }
    }

    /// Set builder code to load from img
    pub fn load<P: AsRef<Path>>(mut self, img_path: P) -> std::io::Result<Self> {
        let img_path: &Path = img_path.as_ref();
        if !img_path.exists() {
            //that's bad, very bad
            return Err(std::io::ErrorKind::NotFound.into());
        }
        let mut f = File::open(img_path)?;
        let mut b: Vec<u8> = vec![];
        f.read_to_end(&mut b)?;
        self.code = b;
        Ok(self)
    }

    /// write img previously loaded by self.load
    fn write_img(&mut self) -> std::io::Result<()> {
        //
        let guest_phys_addr  = self.guest_phys_addr.expect("Can't load image before setting guest_phys_addr");
        unsafe {
            debug!("Writting img");
            let mut vm_mem = slice::from_raw_parts_mut(self.load_addr as *mut u8, self.code.len());
            let written = vm_mem.write(&self.code)?;
            debug!("{written}B written");
            // print 32 instructions just curious
            let mut vm_mem = slice::from_raw_parts_mut(self.load_addr as *mut u8, 0x20);
            vm_mem.disasm_all(self.load_addr as u64);
        }
        Ok(())
    }

    pub fn kvm_allocate_region(
        &self,
        slot: u32,
        userspace_addr: Option<u64>,
        guest_phys_addr: u64,
        size: u64) -> Result<u64> 
    {
        let vm_fd = &self.vm_fd.as_ref().expect("Attempt to allocate kvm mem region on a non initialized VM");
        debug!("KVM Allocation for 0x{size:x} bytes @ guest:0x{guest_phys_addr:x} slot 0");
        let userspace_addr: u64 = unsafe {
            match userspace_addr {
                Some(addr) => addr,
                None =>  
                libc::mmap(
                    null_mut(),
                    size as usize,
                    libc::PROT_READ | libc::PROT_WRITE,
                    libc::MAP_ANONYMOUS | libc::MAP_SHARED | libc::MAP_NORESERVE,
                    -1,
                    0
                ) as u64
            }
        };
        if (userspace_addr as *mut u8).is_null() {
            panic!("mmap returned null ptr");
        }

        debug!("Addr: {:x?}", userspace_addr as *mut u8);
        let mem_region = kvm_userspace_memory_region {
            slot,
            userspace_addr,
            memory_size: size,
            guest_phys_addr,
            flags: KVM_MEM_LOG_DIRTY_PAGES
        };
        unsafe { vm_fd.set_user_memory_region(mem_region)? };
        Ok(userspace_addr as u64)
    }
}

pub trait BuildVm {
    fn setup_vm(&self) -> Result<VmBuilder>;
}

/// Wrapper around VM Creation for KVM, intended to refactor code, maybe useless idk
impl BuildVm for Kvm {
    fn setup_vm(&self) -> Result<VmBuilder> {
        let vm_fd = self.create_vm()?;
        let vmb = VmBuilder::default()
            .vm_fd(vm_fd)
            .slot(0);
        Ok(vmb)
    }
}
