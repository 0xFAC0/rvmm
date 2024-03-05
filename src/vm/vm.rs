extern crate vmm_sys_util;

use std::ptr::null_mut;
use std::usize;

use kvm_bindings::{KVM_MEM_LOG_DIRTY_PAGES, kvm_userspace_memory_region};
use kvm_ioctls::{VmFd, VcpuFd};
use log::{debug, error, info};

use crate::mem_inspection::*;
use crate::vm::VCpu;

use super::Vm;

type Result<T> = std::result::Result<T, kvm_ioctls::Error>;

#[allow(dead_code)]
impl Vm {
    // TODO remove both
    pub fn get_vmfd(&self) -> &VmFd { &self.vm_fd  }
    pub fn get_vcpu(&self) -> &VcpuFd { &self.vcpu_fd }

    fn print_code_at_rip(&self, count: usize) -> Result<()> {
        let addr = self.ram.to_host_addr(self.vcpu_fd.get_rip()?) as *const u8;
        debug!("RIP: 0x{addr:x?}");
        unsafe {
            match addr.mem_region(count) {
                Some(mem_region) => {
                    mem_region.disasm_all(addr as u64);
                }
                None => {
                    debug!("Could not read 0x10 bytes @ {addr:x?}");
                }
            }
        }
        Ok(())
    }

    // pub fn setup_serial_port(&self) {
    //     let kvm_serial: kvm_bindings::kvm_
    // }
    /// Map disk to vm guest memory
    // pub fn map_disk<P: AsRef<Path>>(&mut self, disk_path: P) -> std::io::Result<()> {
    //     let disk_path: &Path = disk_path.as_ref();
    //     let mut f = File::open(disk_path)?;
    //     let mut buf: Vec<u8> = vec![];
    //     f.read_to_end(&mut buf)?;
    //     Ok(())
    // }

    pub fn kvm_allocate_region(
        &mut self,
        slot: u32,
        userspace_addr: Option<u64>,
        guest_phys_addr: u64,
        size: u64) -> Result<u64> 
    {
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
        let mem_region = kvm_userspace_memory_region {
            slot,
            userspace_addr,
            memory_size: size,
            guest_phys_addr,
            flags: KVM_MEM_LOG_DIRTY_PAGES
        };
        unsafe { self.vm_fd.set_user_memory_region(mem_region)? };
        Ok(userspace_addr as u64)
    }

    // TODO error handling in case multiple VM are in use, or maybe restart VM Idk
    pub fn run(&mut self) -> Result<bool> {
        // Let's make a tmp IRQFD, a fd to bound to an interrupt in the guest
        // We will use it to periodically try to wake up the VM
        // let evtfd = EventFd::new(EFD_NONBLOCK).map_err(|e| {error!("EventFD new failed {e}"); e})?;
        // self.vm_fd.register_irqfd(&evtfd, 0).map_err(|e| {error!("VM Register EventFD failed {e}"); e})?;
            // self.print_code_at_rip(0xf)?;
            // evtfd.write(1)?;
        // unsafe { 
        //     let irq: kvm_interrupt = kvm_interrupt { irq: 5 };
        //     debug!("{:x?}", &irq as *const _ as u64);
        //     // 0x86 KVM_INTERRUPT src/include/uapi/linux/kvm.h:1596
        //     let ret = ioctl(self.vcpu_fd.as_raw_fd(), 0x86, &irq as *const _ as u64);
        //     if ret <= 0 {
        //         warn!("KVM_INTERRUPT RET {ret}");
        //         warn!("{}", errno_result::<i32>().expect_err("error no"));
        //     } else {
        //         debug!("KVM_INTERRUPT Injected");
        //     }
        // }
        let opt_vcpu_exit = self.vcpu_fd.run();
        match self.vcpu_fd.run() {
            Err(e) => {
                // If run() fail, VM must be broken, exit anyway
                error!("vCpu exit failed to return");
                self.crash_report(e.to_string().as_str());
                return Err(e);
            },
            Ok(vcpuexit) => {
                // 0xf max size for an operation
                // Should return if vm should continue fetching vcpu run (exiting if not)
                match vcpuexit.handle(&self) {
                    Ok(continue_running) => {
                        if !continue_running {
                            info!("Shutting down after {:?} received", vcpuexit);
                            return Ok(false);
                        }
                    },
                    Err(e) => {
                        error!("Shutting down after {:?} received ({e})", vcpuexit);
                        return Err(e);
                    }
                };
            }
        };
        if let Err(e) = opt_vcpu_exit {
            // TODO TOFIX
            self.crash_report(format!("{e}").as_str());
        } 
        Ok(true)
    }

    // TODO
    pub fn crash_report(&self, e: &str) {
        let sreg = self.vcpu_fd.get_sregs().expect("Could not get special registers while handling error");
        let reg = self.vcpu_fd.get_regs().expect("Could not get special registers while handling error");
        error!("vCPU run failed {e}");
        error!("Special registers:\n{:x?}",sreg);
        error!("Registers:\n{:x?}", reg);
        error!("Code:");
        self.print_code_at_rip(0x20).map_err(|e| error!("could not print code @ RIP {e:x?}")).ok();
    }
}

trait VcpuExitHandle {
    // TODO make errors
    fn handle(&self, vm: &Vm) -> Result<bool>;
}

#[allow(unused)]
impl<'a> VcpuExitHandle for kvm_ioctls::VcpuExit<'a> {
    // bool => continue running vm
    fn handle(&self, vm: &Vm) -> Result<bool> {
        match self {
            Self::IoIn(addr, data) => debug!("{self:x?}"),
            Self::IoOut(addr, data) => debug!("{self:x?}"),
            Self::MmioWrite(addr, data) => { return Ok(true); }, // debug!("{self:x?}"),  
            Self::MmioRead(addr, data) => { return Ok(true); },  // debug!("{self:x?}"),
            Self::Hlt => {
                debug!("exit handler: Hlt reached");
                return Ok(false);
            },
            Self::InternalError => {
                error!("exit handler: InternalError, stay strong");
                vm.crash_report("Internal Error");
                return Ok(false);
            },
            Self::Intr => {
                error!("{self:?} CATCHED YAHOUU");
                todo!()
            },
            Self::Shutdown => {
                debug!("exit handler: Shutdown recv");
                return Ok(false);
            }
            _ => {
                error!("{self:x?} not yet implemented");
                todo!()
            }
        };
        Ok(true)
    }
}

#[allow(unused)]
#[derive(Debug, thiserror::Error, displaydoc::Display)]
pub enum VmmError {
    /// Error while creating the Vm
    VmCreate,
    /// Error while configuring Vm's vCpu
    VcpuConfigure,
    /// Error while exiting vCpu
    VcpuExit,
    /// Error while resuming vCpu
    VcpuResume,
}
