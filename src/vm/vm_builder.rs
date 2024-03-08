use core::slice;
use std::{io::{Read, Write}, path::Path, fs::File, usize, u32};

use goblin::Object;
use kvm_ioctls::{VmFd, Kvm, VcpuFd};
#[allow(unused)]
use log::{debug, error, info, warn};

use crate::mem_inspection::DisASM;

use super::{Vm, ram::{BuildRam, Ram}, serial::SerialPort};

type Result<T> = std::result::Result<T, kvm_ioctls::Error>;

#[allow(dead_code)]
#[derive(Debug)]
pub struct VmBuilder {
    slot: u32,
    vm_fd: VmFd,
    vcpu_fd: VcpuFd,
    code: Vec<u8>,
    ram: Option<Ram>,
    serial: SerialPort,
}

fn find_entrypoint(code: &[u8]) -> u64 {
    for i in 0..code.len() {
        match Object::parse(&code[i..code.len()]).expect("Parsing failed") {
            Object::Elf(_) => todo!(),
            Object::PE(pe) => {
                info!("UEFI Shell PE parsing done");
                return pe.entry as u64;

            },
            Object::COFF(_) => {
                info!("COFF Header found !");
                continue;
            },
            Object::Mach(_) => todo!(),
            Object::Archive(_) => todo!(),
            Object::Unknown(_) => { continue; },
            _ => todo!(),
        }
    }
    todo!()
}

#[allow(unused)]
impl VmBuilder {
    pub fn build(mut self) -> Result<Vm> {
        if self.code.len() == 0 {
            error!("No code loaded, can't run the VM without code. I decided to not build it");
            panic!("Attempt to build VM without code");
        }

        let load_addr = match self.ram.as_ref() {
            None => panic!("Can't start VM without RAM"),
            // TMP, load_addr as entrypoint for code, not realy load addr
            Some(ram) => ram.load_addr + 0x1000
        };

        unsafe {
            // Warning check if 0x1000 + code.len() > mem_size
            let mut vm_mem = slice::from_raw_parts_mut(load_addr as *mut u8, self.code.len());
            let written = vm_mem.write(&self.code).expect("Could not write code");
            let mut vm_mem = slice::from_raw_parts_mut(load_addr as *mut u8, self.code.len());
            vm_mem.disasm_all(0x1000);
            debug!("{written} bytes written");
        }

        // 5. Initialize general purpose and special registers.
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        {
            // x86_64 specific registry setup.
            let mut vcpu_sregs = self.vcpu_fd.get_sregs().unwrap();
            vcpu_sregs.cs.base = 0;
            vcpu_sregs.cs.selector = 0;
            self.vcpu_fd.set_sregs(&vcpu_sregs).unwrap();

            let mut vcpu_regs = self.vcpu_fd.get_regs().unwrap();
            vcpu_regs.rip = 0x1000;
            // vcpu_regs.rax = 2;
            // vcpu_regs.rbx = 3;
            // vcpu_regs.rflags = 2;
            self.vcpu_fd.set_regs(&vcpu_regs).unwrap();
            debug!("RIP set at 0x{:x}", vcpu_regs.rip);
        }

        debug!("{:x?}", self.vcpu_fd);
        Ok( Vm {
            slot: self.slot,
            vm_fd: self.vm_fd, 
            vcpu_fd: self.vcpu_fd,
            ram: self.ram.expect("Can't make VM Without RAM"),
            serial: self.serial
        })
    }

    pub fn ram(mut self, mem_size: usize) -> Self {
        // https://github.com/tianocore/edk2/tree/master/OvmfPkg
        // 1MB OVMF Image memory layout

        let vtf0_size = 0x34000;
        let firmware_image_size_start = mem_size - vtf0_size;
        let ram = self.vm_fd.
            create_ram(mem_size)
            .add_region(0x00, 0xe000) // Non-volatile variable storage
            .add_region(0xe000, 0x1000) // event log area
            .add_region(0xf000, 0x1000) // FTW (fault tolerant write) Work block
            .add_region(0x10000, 0x10000)  // FTW Spare blocks
            .add_region(0x20000, firmware_image_size_start) // Compressed main firmware image 
            .add_region((mem_size - vtf0_size) as u64, vtf0_size) // VTF0 and OVMF SEC
            .build();
        self.ram = Some(ram);
        self
    }

    pub fn load_asm(mut self, asm_code: &'static [u8]) -> Self {
        if asm_code.len() == 0 {
            panic!("can't load null asm code");
        }
        self.code = Vec::from(asm_code);
        self
    }

    /// Set builder code to load from img
    /// Maybe load at addr instead ?
    // pub fn load<P: AsRef<Path>>(mut self, img_path: P, addr: u64) -> std::io::Result<Self> {
    pub fn load<P: AsRef<Path>>(mut self, img_path: P) -> std::io::Result<Self> {
        // let ram: &mut Ram = self.ram.as_mut().expect("You must set RAM before trying to load code somewhere");
        // if !ram.guest_mem_map.address_in_range(GuestAddress(addr)) {
        //     panic!("Address not in range of GPA: 0x{addr:x}");
        // }

        let img_path: &Path = img_path.as_ref();
        info!("loading {}", img_path.to_string_lossy());
        let mut f = File::open(img_path)?;
        let mut b: Vec<u8> = vec![];
        f.read_to_end(&mut b)?;
        self.code = b;
        Ok(self)
    }
}

pub trait BuildVm {
    fn setup_vm(&self) -> Result<VmBuilder>;
}

/// Wrapper around VM Creation for KVM, intended to refactor code, maybe useless idk
impl BuildVm for Kvm {
    fn setup_vm(&self) -> Result<VmBuilder> {
        let vm_fd = self.create_vm()?;
        vm_fd.create_irq_chip()?;
        let vcpu_fd = vm_fd.create_vcpu(0)?;
        Ok(VmBuilder {
            slot: 0,
            vm_fd,
            vcpu_fd,
            code: vec![],
            ram: None,
            serial: SerialPort::new(1)
        })
    }
}
