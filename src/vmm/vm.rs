extern crate vmm_sys_util;

use std::usize;

use kvm_bindings::kvm_interrupt;
use kvm_ioctls::{ VmFd, VcpuFd, VcpuExit };
#[allow(unused)]
use log::{ debug, error, info, warn };

use crate::mem_inspection::*;

use super::Vm;
type Result<T> = std::result::Result<T, kvm_ioctls::Error>;

#[allow(dead_code)]
impl Vm {
    // TODO remove both
    pub fn get_vmfd(&self) -> &VmFd {
        &self.vm_fd
    }
    pub fn get_vcpu(&self) -> &VcpuFd {
        &self.vcpu_fd
    }

    fn print_code_at_rip(&self, count: usize) -> Result<()> {
        let addr = self.vcpu_fd.get_regs()?.rip;
        let cs = self.vcpu_fd.get_sregs()?.cs.base;
        let host_addr = self.ram.load_addr + addr;
        debug!("cs={cs:x?},rip=0x{addr:x?} {:x}", (cs << 4) + addr);
        unsafe {
            // TODO use vm-memory, avoid shit overnight code
            match (host_addr as *const u8).mem_region(count) {
                Some(mem_region) => {
                    mem_region.disasm_count(addr, 0x20);
                }
                None => {
                    debug!("Could not read 0x10 bytes @ {addr:x?}");
                }
            }
        }
        Ok(())
    }

    #[allow(unused)]
    pub fn run(&mut self) -> Result<bool> {
        let vcpu_exit = self.vcpu_fd.run().unwrap();
        info!("--------------------------");
        let rip = self.vcpu_fd.get_regs()?.rip;
        let cs_selector = self.vcpu_fd.get_sregs()?.cs.selector as u64;
        let addr = (cs_selector << 4) + rip;
        debug!("vCPUExit={vcpu_exit:x?}");

        debug!("addr=0x{addr:x},cs_selector=0x{:x},rip=0x{rip:x}", cs_selector);
        match vcpu_exit {
            VcpuExit::IoIn(addr, mut data_asked) => {
                // TOFIX
                debug!("IoIn[0x{addr:x}, {data_asked:x?}]");
                //data_asked.fill(self.serial.data_in());
                data_asked.fill(0);
            }
            VcpuExit::IoOut(addr, data_given) => {
                //
                // From reverse, 0x200 seems to be a IO port which sends strings
                //print!("{}", String::from_utf8_lossy(data_given));
                if [0x200, 0x3f8, 0x402].contains(&addr) || (0x278..=0x27a).contains(&addr) {
                    println!("IO[{addr:x?}] {data_given:x?}");
                    self.serial.data_out(data_given);
                }
            }
            VcpuExit::MmioWrite(addr, data) => {
                debug!("MmioWrite addr=0x{addr:x?} {data:x?}");
                println!("MmioWrite addr=0x{addr:x?} {data:x?}");
            }
            VcpuExit::MmioRead(addr, data) => debug!("MmioWrite 0x{addr:x?} {data:x?}"),
            VcpuExit::Hlt => {
                self.vcpu_fd.get_vcpu_events().unwrap().interrupt.injected = 0;
                #[deprecated(
                    note = "Hardcoded interrupt for HLT, meaningless, remember to study this"
                )]
                return Ok(false);
            }
            VcpuExit::InternalError => {
                self.crash_report("Internal Error");
                return Ok(false);
            }
            VcpuExit::Intr => {
                error!("{self:?} CATCHED YAHOUU");
                //self.vm_fd.set_irq_line(0, active=true).unwrap();
                todo!();
            }
            VcpuExit::Shutdown => {
                return Ok(false);
            }
            VcpuExit::Exception => {
                error!("EXCEPTION {:x?}", self.vcpu_fd.get_vcpu_events().unwrap().exception);
                panic!("EXCEPTION {:x?}", self.vcpu_fd.get_vcpu_events().unwrap().exception);
            }
            _ => {
                error!("{self:x?} not yet implemented");
                todo!();
            }
        }
        Ok(true)
    }

    pub fn crash_report(&self, e: &str) {
        let sreg = self.vcpu_fd
            .get_sregs()
            .expect("Could not get special registers while handling error");
        let reg = self.vcpu_fd
            .get_regs()
            .expect("Could not get special registers while handling error");
        error!("vCPU run failed {e}");
        error!("Special registers:\n{:x?}", sreg);
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
