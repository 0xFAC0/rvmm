use core::slice;
use std::{
    fs::{ File, OpenOptions },
    io::{ stdin,Read, Write },
    path::Path,
};

use goblin::Object;
use kvm_ioctls::{ Kvm, VcpuFd, VmFd };
#[allow(unused)]
use log::{ debug, error, info, warn };

use super::{ ram::{ BuildRam, Ram }, serial::SerialPort, Vm };
use std::thread;
use std::time::Duration;

type Result<T> = std::result::Result<T, kvm_ioctls::Error>;

#[allow(dead_code)]
#[derive(Debug)]
pub struct VmBuilder {
    slot: u32,
    vm_fd: VmFd,
    vcpu_fd: VcpuFd,
    code: Vec<u8>,
    ram: Option<Ram>,
    //serial: Box<dyn SerialPort>,
    serial: SerialPort,
}

pub fn find_entrypoint(firmware_code: &[u8]) -> u64 {
    // Find entrypoint using goblin crate
    for i in 0..firmware_code.len() {
        match Object::parse(&firmware_code[i..]) {
            Ok(Object::PE(pe)) => {
                info!("PE found @ 0x{:x?}", i);
                return (pe.image_base + pe.entry) as u64;
            }
            Ok(Object::COFF(_)) => {
                info!("COFF found @ 0x{:x?}", i);
                let pe = goblin::pe::PE::parse(&firmware_code[i..]).unwrap();
                return (pe.image_base + pe.entry) as u64;
            }
            Ok(Object::Elf(elf)) => {
                info!("ELF found @ 0x{:x?}", i);
                return elf.entry as u64;
            }
            Ok(_) => {}
            Err(e) => {
                error!("Goblin error i=0x{i:x} e={e:x?}");
            }
        }
    }
    error!("No entrypoint found");
    panic!("No entrypoint found");
}

#[allow(unused)]
impl VmBuilder {
    pub fn build(mut self) -> Result<Vm> {
        if self.code.is_empty() {
            error!("No code loaded, can't run the VM without code. I decided to not build it");
            panic!("Attempt to build VM without code");
        }

        let ram = self.ram.as_ref().unwrap();
        let load_addr = ram.load_addr as u64;
        let firmware_load_addr = load_addr + ((ram.mem_size - self.code.len()) as u64);
        unsafe {
            let mut vm_mem = slice::from_raw_parts_mut(load_addr as *mut u8, self.code.len());
            let bytes_written = vm_mem.write(&self.code).expect("Could not write code");
            debug!("{bytes_written} written @ 0x{:x?}", firmware_load_addr);
        }

        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        {
            /**
            let mut vcpu_sregs = self.vcpu_fd.get_sregs().unwrap();

            vcpu_sregs.cs.base = 0;
            vcpu_sregs.cs.selector = 0;
            vcpu_sregs.cs.limit = 0xffff_ffff;

            vcpu_sregs.ds.base = 0;
            vcpu_sregs.ds.limit = 0xffffffff;
            vcpu_sregs.ds.selector = 0;

            vcpu_sregs.es = vcpu_sregs.ds;
            vcpu_sregs.fs = vcpu_sregs.ds;
            vcpu_sregs.gs = vcpu_sregs.ds;
            vcpu_sregs.ss = vcpu_sregs.ds;
            self.vcpu_fd.set_sregs(&vcpu_sregs).unwrap();
            */

            let mut vcpu_regs = self.vcpu_fd.get_regs().unwrap();
            //vcpu_regs.rip = 0xffff_ffff_ffff_fff0; // Reset Vector
            vcpu_regs.rip = (ram.mem_size - self.code.len()) as u64;
            vcpu_regs.rflags = 0x2;
            debug!("set regs: rip=0x{:x?}, rflags=0x{:x?}", vcpu_regs.rip, vcpu_regs.rflags);
            self.vcpu_fd.set_regs(&vcpu_regs).unwrap();
        }

        thread::sleep(Duration::from_secs(3));
        Ok(Vm {
            slot: self.slot,
            vm_fd: self.vm_fd,
            vcpu_fd: self.vcpu_fd,
            ram: self.ram.expect("Can't make VM Without RAM"),
            serial: self.serial,
        })
    }

    pub fn ram(mut self, mem_size: usize) -> Self {
        let ram = self.vm_fd.create_ram(mem_size).build();
        self.ram = Some(ram);
        self
    }

    pub fn load_asm(mut self, asm_code: &'static [u8]) -> Self {
        if asm_code.is_empty() {
            panic!("can't load null asm code");
        }
        self.code = Vec::from(asm_code);
        self
    }

    /// Set builder code to load from img
    /// Maybe load at addr instead ?
    pub fn load<P: AsRef<Path>>(mut self, img_path: P) -> std::io::Result<Self> {
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
        // TMP TODO REMOVE
        let path = "/tmp/vmm.serial";
        let file: File = OpenOptions::new().read(true).write(true).create(true).open(path).unwrap();
        #[allow(unused)]
        let fd_in = Box::new(file.try_clone().unwrap());
        #[allow(unused)]
        let fd_out = Box::new(file);
        // TMP TODO REMOVE
        let vm_fd = self.create_vm()?;
        vm_fd.create_irq_chip()?;
        let vcpu_fd = vm_fd.create_vcpu(0)?;

        Ok(VmBuilder {
            slot: 0,
            vm_fd,
            vcpu_fd,
            code: vec![],
            ram: None,
            // TODO VM Builder args
            //serial: SerialPort::new(0x38f, fd_in, fd_out),
            serial: SerialPort::new(0x38f, Box::new(stdin()), fd_out),
        })
    }
}
