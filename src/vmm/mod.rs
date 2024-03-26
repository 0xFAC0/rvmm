use kvm_ioctls::{VcpuFd, VmFd};
use self::serial::SerialPort;

use self::ram::Ram;

pub mod vm_builder;
pub mod vm;
pub mod ram;
pub mod serial;

#[allow(dead_code)]
#[derive(Debug)]
pub struct Vm {
    slot: u32,
    vm_fd: VmFd,
    vcpu_fd: VcpuFd,
    pub ram: Ram,
    pub serial: SerialPort
}

#[allow(dead_code)]
/// VCpu wrapper around VCpuFd implemmenting common Vcpu getter/setter and operations
pub trait VCpu {
    // Wrapper to get most used registers
    fn get_rip(&self) -> Result<u64, kvm_ioctls::Error>;
}

impl VCpu for VcpuFd {
    fn get_rip(&self) -> Result<u64, kvm_ioctls::Error> {
        Ok(self.get_regs()?.rip)
    }
}
