use super::{DeviceConfigError,ByteDevice};

use std::collections::{HashMap,VecDeque};
use std::error;
use std::io::{Read,Write};
use std::ops::Range;
use std::sync::{Arc,Mutex};
use std::sync::mpsc::{channel,Sender,SendError};
use std::thread::{JoinHandle,self};

use serialport::{SerialPort,TTYPort};
use thiserror::Error;

// How much of the code for this should be in the backend?
// I'm not sure how much is reusable, probably will have to reorganize eventually
enum SerialBackend {
    PTY(PTYBackend)
}
struct PTYBackend {
    writer: Sender<u8>,
    _port: TTYPort,
    _write_thread: JoinHandle<()>,
    _read_thread: JoinHandle<()>
}
impl PTYBackend {
    fn write_byte(&self,data: u8) -> Result<(),SerialError> {
        Ok(self.writer.send(data)?)
    }
    fn new(read_buffer: Arc<Mutex<VecDeque<u8>>>) -> Result<Self,DeviceConfigError> {
        // First, find an openable PTY
        let (mut port, _slave) = TTYPort::pair()//serialport::new("/dev/ttyS10", 115200)
            .map_err(|e| 
                     DeviceConfigError::Custom(format!("Failed to open PTY Master with error {e}")))?;

        print!("\r\nTTY opened at {:?}\r\n",_slave.name());
        // NOTE: cloning everything this way means that modifying serial port settings will not
        // work. This should get changed eventually but doesnt matter for now.
        let port_reader = port.try_clone_native()
            .map_err(|e| DeviceConfigError::Custom(format!("{e}")))?;

        let master_port = port.try_clone_native()
            .map_err(|e| DeviceConfigError::Custom(format!("{e}")))?;
        //let mut file: Option<File> = None;
        //for num in 0..10 {
        //    match 
        //        // I really dont think you're allowed to open pty masters multiple times ....
        //        // welp BSD style ttys are way deprecated, only 20 years too late rip
        //        OpenOptions::new().read(true).write(true).open(format!("/dev/ptyS1{num}")) {
        //        Ok(w) => {
        //            file = Some(w); 
        //            break
        //        },
        //        e => {eprintln!("{:?}",e); continue}
        //    }
        //}
        //let mut file = match file {
        //    Some(f) => f,
        //    None => return Err(DeviceConfigError::Custom("Failed to open PTY Master".to_string()))
        //};
        // Not sure if I should do this cursed cloning or if I can open the tty multiple times ....
        // This might unintentionally close the pty maybe? But if either the reader or writer
        // breaks its probably good to crash both
        //let fd = file.into_raw_fd();
        //let file_for_reader = unsafe {File::from_raw_fd(fd)};
        //let mut file_for_writer = unsafe {File::from_raw_fd(fd)};

        let (writer, reader) = channel();

        // This thread lets us do nonblocking writes and reads
        let write_thread = thread::spawn(move || {
            for byte in reader.iter() {
                // Crash if something goes wrong, no better way to handle errors from child threads
                // yet
                port.write(&[byte]).expect("Failed to write to TTY, panicking");
            }
        });
        let read_thread = thread::spawn(move || {
            for byte in port_reader.bytes() {
                let byte = byte.expect("Unable to read from PTY, panicking");
                read_buffer.lock().expect("PTY reader poisoned").push_back(byte);
            }

        });

        Ok(PTYBackend {
            writer,
            _port: master_port,
            _write_thread: write_thread,
            _read_thread: read_thread,
        })


    }
}
pub struct Serial {
    // Defalts to 0x3F8 but can be overridden
    base_address: u32,
    backend: SerialBackend,
    read_buffer: Arc<Mutex<VecDeque<u8>>>
    
}
impl Serial {
    pub fn from_config(options: HashMap<String,String>) -> Result<Box<Serial>,DeviceConfigError> {
        let base_address = match options.get("address") {
            Some(a) => u32::from_str_radix(a,10)?,
            None => 0x3F8
        };
        let read_buffer = Arc::new(Mutex::new(VecDeque::new()));
        let backend = match options.get("backend").map(|s| s.as_str()) {
            None | Some("pty") => SerialBackend::PTY(PTYBackend::new(read_buffer.clone())?),
            Some(s) => return Err(DeviceConfigError::InvalidOption(s.to_string()))

        };
        Ok(Box::new(Serial {
            base_address,
            backend,
            read_buffer
        }))

    }

}
impl ByteDevice for Serial {
    fn memory_range(&self) -> Range<u32> {
        self.base_address .. (self.base_address + 8)

    }
    fn store_byte(&self, addr: u32, data: u8) -> Result<(),Box<dyn error::Error>> {
        let offset = addr - self.base_address;
        // Write to the character device
        // Other registers here arent implemented
        if offset == 0 {
            match &self.backend {
                SerialBackend::PTY(p) => Ok(p.write_byte(data)?),
            }
        } else {
            // Dummy return value
            Ok(())
        }

    }
    fn read_byte(&self, addr: u32) -> Result<i8,Box<dyn error::Error>> {
        let offset = addr - self.base_address;
        // Line status registor
        if offset == 5 {
            let can_read = if self.read_buffer.lock().map_err(|_| SerialError::ThreadPoisonError)?.len() != 0 {
                1
            } else {
                0
            };
            Ok(1 << 5 & can_read)
        } else if offset == 0 {
            let deque = &mut self.read_buffer.lock().map_err(|_| SerialError::ThreadPoisonError)?;
            // If there is no data in the buffer then return dummy data
            match deque.pop_front() {
                None => Ok(0),
                Some(data) => Ok(data as i8)
            }

        } else {
            Ok(0)
        }
    }

}

#[derive(Debug,Error)]
pub enum SerialError {
    #[error("Error communicating with child thread")]
    ThreadSendError(#[from] SendError<u8>),
    #[error("Child thread panicked")]
    ThreadPoisonError,

}
