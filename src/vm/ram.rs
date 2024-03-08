use std::ptr::null_mut;

use kvm_bindings::{kvm_userspace_memory_region, KVM_MEM_LOG_DIRTY_PAGES};
use kvm_ioctls::VmFd;
use log::{debug, warn};
use vm_memory::{GuestMemoryMmap, GuestAddress};

#[derive(Debug)]
#[allow(unused)]
pub struct Ram {
    pub load_addr: u64,
    pub mem_size: usize,
    pub guest_phys_addr: u64,       
    pub guest_mem_map: GuestMemoryMmap,
}

pub trait BuildRam {
    fn create_ram(&self, mem_size: usize) -> RamBuilder;
}

impl BuildRam for VmFd {
    fn create_ram(&self, mut mem_size: usize) -> RamBuilder {
        debug!("making RAM with size: 0x{mem_size:x}");
        // Try to align
        if mem_size % 0x10 != 0 {
            debug!("Unaligned memory size: {mem_size}");
            mem_size += mem_size % 0x10;
        }
        RamBuilder {vm_fd: &self, host_load_addr: None, mem_size, regions: vec![] }
    }
}

#[derive(Debug)]
#[allow(unused)]
pub struct RamBuilder<'a> {
    vm_fd: &'a VmFd,
    /// Host userspace address where the guest memory is allocated. Not GPA
    host_load_addr: Option<u64>,
    mem_size: usize,
    regions: Vec<(GuestAddress, usize)>, // Vec<(addr_start, size)>
}

#[allow(unused)]
impl<'a> RamBuilder<'a> {
    pub fn add_region(mut self, start: u64, size: usize) -> Self {
        if size == 0 {
            warn!("RamBuilder: adding a region with null size: start=0x{start:x?},size=0 ");
        }
        self.regions.push((GuestAddress(start), size));
        self
    }

    pub fn build(self) -> Ram {
        // let gm = GuestMemoryMmap::<()>::from_ranges(self.regions.as_ref())
        //     .expect("Could not create guest memory");
        // Careful, slot must change
        let host_userspace_addr = self.kvm_allocate_region(0, None, 0, self.mem_size as u64);
        Ram {
            load_addr: host_userspace_addr,
            mem_size: self.mem_size,
            guest_phys_addr: 0,
            guest_mem_map: GuestMemoryMmap::new() 
        }
    }

    fn kvm_allocate_region(
        &self,
        slot: u32,
        userspace_addr: Option<u64>,
        guest_phys_addr: u64,
        size: u64) -> u64
    {
        let vm_fd = &self.vm_fd;
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
        if userspace_addr <= 0 {
            panic!("Mmap failed (TODO errno) ret={userspace_addr}");
        }

        debug!("Addr: {:x?}", userspace_addr as *mut u8);
        let mem_region = kvm_userspace_memory_region {
            slot,
            userspace_addr,
            memory_size: size,
            guest_phys_addr,
            flags: KVM_MEM_LOG_DIRTY_PAGES
        };
        unsafe { vm_fd.set_user_memory_region(mem_region).unwrap() }
        userspace_addr
    }
}
