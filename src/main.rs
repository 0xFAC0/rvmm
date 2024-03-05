extern crate kvm_bindings;

use clap::Parser;
use kvm_ioctls::Kvm;

mod args;
mod asm_code;
mod mem_inspection;
mod vm;
// not ready
//mod devices;
mod vcpu;

use crate::args::{Cli, Verbosity};
use crate::vm::vm_builder::*;

#[allow(unused)]
use log::{debug, error, info, warn};

fn setup_logging(verbosity: Verbosity) {
    // Don't overwrite already existing RUST_LOG var
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", verbosity.to_string());
    }
    env_logger::init();
}

fn main() {
    let cli = Cli::parse();
    setup_logging(cli.verbosity);
    debug!("env_logger init");
    info!("--- Fuck Vanguard Starting ---");

    let kvm: Kvm = Kvm::new().expect("KVM Failed to start");
    let mut vm = kvm
        .setup_vm()
        .expect("KVM Create VM failed")
        .slot(0)
        .mem_size(1024 * 1024 * 1024)
        .guest_phys_addr(0)
        //.asm_code(crate::asm_code::asm_mmio_rw_test())
        .load("/mnt/bootmgr.efi").expect("img failed to load")
        .build()
        .expect("VM Creation failed");
    info!("Starting VM");


    loop {
        match vm.run() {
            Ok(keep_running) => { if !keep_running {break;} },
            Err(_) => { break; }
        }
    }
    // let vmexit_count: usize = 0;
    // let vmexit_count_th = Arc::new(Mutex::new(vmexit_count));
    // let vmexit_count = vmexit_count_th.clone();
    // let th_handle = thread::spawn(move || {
    //     debug!("Child thread handle");
    //     vm.load("/mnt/bootmgr.efi").unwrap();
    //     loop {
    //         if let Ok(mut _vmexit_count) = vmexit_count_th.try_lock() {
    //             match vm.run() {
    //                 Ok(keep_running) => {
    //                     if !keep_running {
    //                         break;
    //                     }
    //                 },
    //                 Err(_) => { break; }
    //             }
    //         }
    //     }
    // });
    
    // let mut t1 = Instant::now();
    // let mut hanging = false;
    // loop {
    //     if th_handle.is_finished() {
    //         info!("VMs terminated, quitting");
    //         break;
    //     }

    //     if let Ok(_vmexit_count) = vmexit_count.try_lock() {
    //         t1 = Instant::now();
    //         if hanging {
    //             // _vm.get_vcpu().nmi().unwrap();
    //             hanging = false;
    //         }
    //         // _vm.crash_report("test");
    //     }
    //     if t1.elapsed() > Duration::from_secs(4) {
    //         if !hanging {
    //             warn!("VM Seems to hang");
    //             hanging = true;
    //         }
    //     }
    //     if t1.elapsed() > Duration::from_secs(10) {
    //         error!("Critical hang, quitting");
    //         th_handle.kill(0).unwrap();
    //         break;
    //     }
    // }
    // //vm.start().map_err(|e| error!("VM end with {e}")).ok();
    // info!("Quitting");
}
