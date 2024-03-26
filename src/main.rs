extern crate kvm_bindings;

use std::time::SystemTime;

use clap::Parser;
use kvm_ioctls::Kvm;

mod args;
mod asm_code;
mod mem_inspection;
mod vmm;

use crate::args::{ Cli, Verbosity };
use crate::vmm::vm_builder::*;

#[allow(unused)]
use log::{ debug, error, info, warn };

fn setup_logging(
    verbosity: Verbosity,
    log_file: Option<&'static str>
) -> Result<(), fern::InitError> {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", verbosity.to_string());
    }
    let log_destination: fern::Output = if let Some(path) = log_file {
        fern::log_file(path)?.try_into().unwrap()
    } else {
        std::io::stdout().try_into().unwrap()
    };

    fern::Dispatch
        ::new()
        .format(|out, msg, record| {
            out.finish(
                format_args!(
                    "[{} {} {}] {}",
                    humantime::format_rfc3339_seconds(SystemTime::now()),
                    record.level(),
                    record.target(),
                    msg
                )
            )
        })
        .level(log::LevelFilter::Debug)
        .chain(log_destination)
        .apply()?;
    Ok(())
}

fn main() {
    let cli = Cli::parse();
    setup_logging(cli.verbosity, Some("/tmp/vmm.log")).unwrap();
    debug!("logger init done");
    info!("--- Fuck Vanguard Starting ---");

    let kvm: Kvm = Kvm::new().expect("KVM Failed to start");
    let mut vm = kvm
        .setup_vm()
        .expect("KVM Create VM failed")
        .ram(0x100000000) // 4GB (see OVMF doc)
        .load("/home/paco/repo/edk2/Build/OvmfX64/DEBUG_GCC5/FV/OVMF.fd")
        .unwrap()
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
