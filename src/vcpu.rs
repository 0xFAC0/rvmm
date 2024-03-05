use kvm_ioctls::VcpuFd;

/// Abstraction over common CPU operations
pub trait VCpu {
    fn get_rip(&self) -> Result<u64, kvm_ioctls::Error>;
}

impl VCpu for VcpuFd {
    fn get_rip(&self) -> Result<u64, kvm_ioctls::Error> {
        Ok(self.get_regs()?.rip)
    }
}
