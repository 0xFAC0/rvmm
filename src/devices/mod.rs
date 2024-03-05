extern crate kvm_bindings;

use crate::vm::Vm;

use kvm_ioctls::{DeviceFd, VmFd};
use kvm_bindings::bindings::*;

// struct Device {
//     fd: DeviceFd,
// }

pub trait Device {}

pub trait MakeDevice {
    fn new_device(&self, device_type: kvm_bindings::bindings::kvm_device_type) -> Result<Box<dyn Device>, kvm_ioctls::Error>; 
}

impl MakeDevice for Vm {
    fn new_device(&self, device_type: kvm_device_type) -> Result<Box<dyn Device>, kvm_ioctls::Error> {
        let fd = 0;
        let device: Box<dyn Device> = match device_type {
            KVM_DEV_TYPE_KEYBOARD => Box::new(KeyboardDevice::new( &self.get_vmfd(), fd)),
            KVM_DEV_TYPE_MOUSE => Box::new(MouseDevice::new(&self.get_vmfd(), fd)),
            // kvm_device_type
            _ => todo!(),
        };

        Ok(device)
    }
}

// KVM_DEV_TYPE_KEYBOARD
struct KeyboardDevice {
    fd: DeviceFd,
}

impl Device for KeyboardDevice {}
impl KeyboardDevice {
    pub fn new(vmfd: &VmFd, fd: usize) -> Self {
        todo!()
    }
}

// KVM_DEV_TYPE_MOUSE
struct MouseDevice {
    fd: DeviceFd,
}

impl Device for MouseDevice {}
impl MouseDevice {
    pub fn new(vmfd: &VmFd, fd: usize) -> Self {
        todo!()
    }
}

// KVM_DEV_TYPE_SERIAL 
struct SerialDevice {
    fd: DeviceFd,
}

impl SerialDevice {
    pub fn new(vmfd: &VmFd, fd: usize) -> Self {
        todo!()
    }
}

struct DiskDevice {
    fd: DeviceFd,
}

impl DiskDevice {
    pub fn new(vmfd: &VmFd, fd: usize) -> Self {
        todo!()
    }
}

