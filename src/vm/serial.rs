use std::{fs::File, io::{stdout, Write}};

use console::Term;
use log::{info, debug};

#[derive(Debug)]
pub struct SerialPort {
    pub port: u32,
    fd_in: File,
    term: Term,
    line_buffer: Vec<u8>,
}

impl SerialPort {
    pub fn new(port: u32) -> Self {
        let fd_in = File::open("/tmp/serial1.vmm").unwrap();
        let term = Term::read_write_pair(fd_in.try_clone().unwrap(), stdout());
        Self {port, fd_in, term, line_buffer: vec![]}
    } 

    /// out instruction for guest
    pub fn data_in(&mut self, data: &[u8]) {
        debug!("recv: {data:x?}");
        self.line_buffer.append(&mut data.to_vec());
        if self.line_buffer.contains(&0x0) {
            info!("{}", String::from_utf8_lossy(self.line_buffer.as_ref()));
        }
        // self.fd_in.write(data).unwrap();
    }

    /// in instruction for guest
    pub fn data_out(&mut self) -> u8 {
        self.term.read_char().unwrap() as u8
    }
}
// impl Drop for SerialPort {
//     fn drop(&mut self) {
//     }
// }
