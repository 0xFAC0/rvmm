use core::slice;

use capstone::prelude::*;

#[allow(unused)]
use log::{debug, info, warn, error};

#[allow(unused, unreachable_code)]
pub fn init_cs_disass_x86_64() -> Capstone {
    #[cfg(target_arch = "x86_64")]
    {
        return Capstone::new()
            .x86()
            .mode(arch::x86::ArchMode::Mode64)
            .syntax(arch::x86::ArchSyntax::Intel)
            .detail(true)
            .build()
            .expect("Failed to create Capstone object");
    }
    todo!()
}

fn disasm_all(code: &[u8], addr: u64) {
    let mut _o_cs: Option<Capstone> = None;

    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    {
        _o_cs = Some(init_cs_disass_x86_64());
    }
    let mut cs = _o_cs.expect("Architecture not supported yet");
    
    //TMP
    cs.set_skipdata(true).expect("How the fuck did CapStone set_skipdata failed");

    // TOFIX
    let insns = cs.disasm_all(code, addr).expect("Disassembly failed");
    
    for i in insns.as_ref() {
        debug!("{} {:x?}", i, i.bytes());
    }
}

pub trait MemRegion {
    unsafe fn mem_region<'a>(&'a self, len: usize) -> Option<&'a [u8]>;
} 

impl MemRegion for *const u8 {
    unsafe fn mem_region<'a>(&'a self, len: usize) -> Option<&'a [u8]> {
        if self.is_null() {
            return None;
        }
        unsafe {
            let mem: &[u8] = slice::from_raw_parts(*self, len);
            debug!("{:x?}", mem);
            Some(mem)
        }
    }
}

pub trait DisASM {
    fn disasm_count(&self, addr: u64, count: usize);
    fn disasm_all(&self, addr: u64);
}

impl DisASM for &[u8] {
    fn disasm_all(&self, addr: u64) { disasm_all(&self, addr); }
    #[allow(unused)]
    fn disasm_count(&self, addr: u64, count: usize) {
        todo!()
    }
}
impl DisASM for &mut [u8] {
    fn disasm_all(&self, addr: u64) { disasm_all(&self, addr); }
    #[allow(unused)]
    fn disasm_count(&self, addr: u64, count: usize) {
        todo!()
    }
}
