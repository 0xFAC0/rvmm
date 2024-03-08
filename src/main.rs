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
        .ram(0x100000000) // 4GB (see OVMF doc)
        .load_asm(include_bytes!("../test_serial.bin"))
        .build()
        .expect("VM Creation failed");
    info!("Starting VM");

    // Todo bring errors up here, only error!() in calling function. panic!() here ?
    while let Ok(keep_running) = vm.run() {
        if !keep_running {
            break;
        }
    }
    info!("Nicely shutdown, well played ;)")
}
