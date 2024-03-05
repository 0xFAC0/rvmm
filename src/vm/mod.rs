use kvm_ioctls::{DeviceFd, VcpuFd, VmFd};

pub mod vm_builder;
pub mod vm;

#[allow(dead_code)]
#[derive(Debug)]
pub struct Vm {
    slot: u32,
    vm_fd: VmFd,
    vcpu_fd: VcpuFd,
    pub ram: Ram,
    devices_fd: Vec<DeviceFd>,
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

#[allow(unused)]
// Might be dangerous to have the field in pub, maybe set getters to inline
// TODO understand TSS
#[derive(Debug)]
pub struct Ram {
    pub load_addr: u64,
    pub mem_size: usize,
    pub guest_phys_addr: u64,
    pub tss_addr: Option<u64>,
}

#[allow(dead_code)]
impl Ram {
    pub fn new(load_addr: u64, mem_size: usize, guest_phys_addr: u64, tss_addr: Option<u64>) -> Self {
        Self {load_addr, mem_size, guest_phys_addr, tss_addr}
    }

    #[inline(always)]
    pub fn mem_size(&self) -> usize {
        self.mem_size
    }

    #[inline(always)]
    pub fn load_addr(&self) -> u64 {
        self.load_addr as u64
    }

    #[inline(always)]
    pub fn guest_phys_addr(&self) -> u64 {
        self.guest_phys_addr as u64
    }

    pub fn to_host_addr(&self, guest_addr: u64) -> u64 {
        // guest_phys_addr is user space memory start thus and load_addr is the host addr holding this
        // memory space thus guest addr is an offset
        self.load_addr as u64 + guest_addr - self.guest_phys_addr
    }
}
