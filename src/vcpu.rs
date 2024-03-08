use kvm_ioctls::VcpuFd;

/// Abstraction over common CPU operations
#[allow(dead_code)]
pub struct VCpu(VcpuFd);
