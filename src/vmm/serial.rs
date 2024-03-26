use std::io::{ Read, Write };
use std::fmt::Debug;

use log::{ debug, error, info };

#[allow(dead_code)]
pub struct SerialPort {
    pub port: u32,
    line_buffer: Vec<u8>,
    fd_in: Box<dyn Read + 'static>,
    fd_out: Box<dyn Write + 'static>,
}

impl SerialPort {
    pub fn new(port: u32, fd_in: Box<dyn Read>, fd_out: Box<dyn Write>) -> Self {
        Self {
            port,
            line_buffer: vec![],
            fd_in,
            fd_out,
        }
    }

    // out instruction for guest
    pub fn data_out(&mut self, data: &[u8]) {
        debug!("recv: {data:x?}");
        self.line_buffer.append(&mut data.to_vec());
        if self.line_buffer.contains(&0x0) {
            info!("Printing to fd: {}", String::from_utf8_lossy(self.line_buffer.as_ref()));
            self.fd_out.write_all(&self.line_buffer).unwrap();
        }
    }

    // TOFIX HARDCODED SIZE
    pub fn data_in(&mut self) -> u8 {
        #[deprecated(note = "Hardcoded magic size for serial IN instruction")]
        let mut buf = [0u8; 1];
        if self.fd_in.read(&mut buf).is_err() {
            error!("Could not read from serial {self:x?}");
        }
        buf.first().unwrap().to_owned()
    }
}

impl Debug for SerialPort {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SerialPort")
            .field("port", &self.port)
            .field("line_buffer", &self.line_buffer)
            .finish()
    }
}
