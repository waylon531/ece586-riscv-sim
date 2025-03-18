use super::{DeviceConfigError,WordDevice,Device};

use minifb::{Window,WindowOptions,Scale};

use std::collections::HashMap;
use std::error;
use std::ops::RangeInclusive;
use std::sync::{Arc,Mutex};
use std::time;
use std::thread::{JoinHandle,self};

use thiserror::Error;

enum FramebufferBackend {
    MiniFB(JoinHandle<()>)
}

const WIDTH: usize = 160;
const HEIGHT: usize = 144;

pub struct Framebuffer {
    base_address: u32,
    backend: FramebufferBackend,
    pixel_buffer: Arc<Mutex<Box<[u32]>>>
    
}
impl Framebuffer {
    pub fn from_config(options: HashMap<String,String>) -> Result<Device,DeviceConfigError> {
        let base_address = match options.get("address") {
            Some(a) => u32::from_str_radix(a,10)?,
            // Default of 0xFF000000
            None => 0xF000000
        };
        let pixel_buffer = Arc::new(Mutex::new(vec![0u32; WIDTH*HEIGHT].into_boxed_slice()));
        let pixel_buffer_copy = pixel_buffer.clone();
        let backend = match options.get("backend").map(|s| s.as_str()) {
            None | Some("minifb") => {
                FramebufferBackend::MiniFB(
                thread::spawn(move || {
                    let pixel_buffer = pixel_buffer_copy;
                    // I feel like I should be able to initialize this outside the thread and move
                    // it in, but whatever
                    let mut options = WindowOptions::default();
                    options.scale = Scale::X4;
                    let mut window = match Window::new(
                        "REMU", 
                        WIDTH, 
                        HEIGHT, 
                        options,
                        ) {
                        Ok(win) => win,
                        Err(err) => {
                            eprintln!("\r\n{}\r\n",err);
                            // Note: I should figure out how to return errors better from devices
                            panic!("Could not initialize framebuffer")
                            //return Err(DeviceConfigError::Custom(format!("{}",err)));
                        }
                    };
                    window.set_target_fps(30);
                    loop {
                        {
                            window.update_with_buffer(
                                &**pixel_buffer.lock().expect("FB poisoned"),WIDTH,HEIGHT
                            ).expect("Unable to write to FB");
                        }
                        // update at around 30fps
                        let thirty_fps_sleep = time::Duration::from_millis(33);
                        thread::sleep(thirty_fps_sleep);

                    }


                }))
            },
            Some(s) => return Err(DeviceConfigError::InvalidOption(s.to_string()))

        };
        Ok(Device::WordDevice(Box::new(Framebuffer {
            base_address,
            backend,
            pixel_buffer
        })))

    }

}
impl WordDevice for Framebuffer {
    fn memory_range(&self) -> RangeInclusive<u32> {
        self.base_address ..= (self.base_address + (32*WIDTH*HEIGHT) as u32)
    }
    // This only supports aligned access
    fn store_word(&self, addr: u32, data: u32) -> Result<(),Box<dyn error::Error>> {
        match self.backend {
            FramebufferBackend::MiniFB(_) => {
                let offset: usize = (addr - self.base_address) as usize >> 2;
                self.pixel_buffer.lock().map_err(|_| FramebufferError::Poisoned )?[offset] = data;
                Ok(())
            }
        }
    }
    fn read_word(&self, addr: u32) -> Result<i32,Box<dyn error::Error>> {
        match self.backend {
            FramebufferBackend::MiniFB(_) => {
                let offset: usize = (addr - self.base_address) as usize >> 2;
                Ok(
                    self.pixel_buffer.lock().map_err(|_| FramebufferError::Poisoned )?[offset] as i32
                  )
            }
        }
    }

}

#[derive(Debug,Error)]
pub enum FramebufferError {
    #[error("Framebuffer poisoned")]
    Poisoned,
}
