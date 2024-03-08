extern crate vmm_sys_util;

use std::usize;

use kvm_ioctls::{VmFd, VcpuFd, VcpuExit};
use log::{debug, error, info};

use crate::mem_inspection::*;

use super::Vm;

type Result<T> = std::result::Result<T, kvm_ioctls::Error>;

#[allow(dead_code)]
impl Vm {
    // TODO remove both
    pub fn get_vmfd(&self) -> &VmFd { &self.vm_fd  }
    pub fn get_vcpu(&self) -> &VcpuFd { &self.vcpu_fd }

    fn print_code_at_rip(&self, count: usize) -> Result<()> {
        let addr = self.vcpu_fd.get_regs()?.rip;
        let host_addr = self.ram.load_addr + addr;
        debug!("RIP: 0x{addr:x?}");
        unsafe {
            // TODO use vm-memory, avoid shit overnight code
            match (host_addr as *const u8).mem_region(count) {
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


    // TODO error handling in case multiple VM are in use, or maybe restart VM Idk
    #[allow(unused)]
    pub fn run(&mut self) -> Result<bool> {
        let vcpu_exit = self.vcpu_fd.run().unwrap();
        debug!("RIP=0x{:x},vCPU Exit handling", self.vcpu_fd.get_regs()?.rip);
        // debug!("LAPIC={:x?}", self.vcpu_fd.get_lapic());
        debug!("VCPU Pending events: {:x?}", self.vcpu_fd.get_vcpu_events()?);
        match vcpu_exit {
            VcpuExit::IoIn(addr, mut data_asked) => {
                debug!("Io In 0x{addr:x?} {data_asked:x?}");
                data_asked = &mut [0x41, 0x41, 0x41, 0x41];
            },
            VcpuExit::IoOut(addr, data_given) => {
                debug!("Io Out 0x{addr:x?} {data_given:x?}");
                debug!("{:x?}", self.vcpu_fd.get_regs()?.rax);
                // self.serial.data_in(data_given);
            },
            VcpuExit::MmioWrite(addr, data) => debug!("MmioWrite 0x{addr:x?} {data:x?}"),
            VcpuExit::MmioRead(addr, data) => debug!("MmioWrite 0x{addr:x?} {data:x?}"),
            VcpuExit::Hlt => {
                debug!("exit handler: Hlt reached");
                return Ok(false);
            },
            VcpuExit::InternalError => {
                error!("exit handler: InternalError, stay strong");
                self.crash_report("Internal Error");
                return Ok(false);
            },
            VcpuExit::Intr => {
                error!("{self:?} CATCHED YAHOUU");
                todo!()
            },
            VcpuExit::Shutdown => {
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

    // TODO
    pub fn crash_report(&self, e: &str) {
        let sreg = self.vcpu_fd.get_sregs().expect("Could not get special registers while handling error");
        let reg = self.vcpu_fd.get_regs().expect("Could not get special registers while handling error");
        error!("vCPU run failed {e}");
        error!("Special registers:\n{:x?}",sreg);
        error!("Registers:\n{:x?}", reg);
        error!("Code:");
        // self.print_code_at_rip(0x20).map_err(|e| error!("could not print code @ RIP {e:x?}")).ok();
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
