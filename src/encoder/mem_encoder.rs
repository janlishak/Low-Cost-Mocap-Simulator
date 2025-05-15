use super::{Encoder, Result};
use bevy::prelude::*;
use shared_memory::ShmemConf;

pub struct MyCustomEncoder {
    shmem: shared_memory::Shmem,
    frame_size: usize,
}

impl MyCustomEncoder {
    /// Creates a new encoder that writes raw RGBA8 frames into shared memory.
    /// If the shared memory exists, it opens the existing mapping.
    pub fn new(name: &str, width: usize, height: usize) -> Self {
        let channels = 4;  // RGBA8 format
        let frame_size = width * height * channels;

        let shmem = match ShmemConf::new()
            .os_id(name)
            .size(frame_size)
            .create()
        {
            Ok(mem) => mem,
            Err(e) => {
                if let shared_memory::ShmemError::MappingIdExists = e {
                    ShmemConf::new()
                        .os_id(name)
                        .open()
                        .unwrap_or_else(|e| panic!("Failed to open existing shared memory '{}': {}", name, e))
                } else {
                    panic!("Failed to create shared memory '{}', size {} bytes: {}", name, frame_size, e);
                }
            }
        };

        println!("buffer created for mem_encoder size {}", frame_size);

        Self { shmem, frame_size }
    }
}

// impl MyCustomEncoder {
//     /// Creates a new encoder that writes raw RGBA8 frames into shared memory.
//     pub fn new(name: &str, width: usize, height: usize) -> Self {
//         let channels = 4;  // Since we use RGBA8 format
//         let frame_size = width * height * channels;

//         let shmem = ShmemConf::new()
//             .os_id(name)
//             .size(frame_size)
//             .create()
//             .unwrap_or_else(|e| panic!("Failed to create shared memory '{}', size {} bytes: {}", name, frame_size, e));

//         Self { shmem, frame_size }
//     }
// }

impl Encoder for MyCustomEncoder {
    fn encode(&mut self, image: &Image) -> Result<()> {
        // Convert Bevy's Image to DynamicImage
        let dynamic_image = image.clone().try_into_dynamic()?;

        // Convert to RGBA8 format and get raw bytes
        let rgba_image = dynamic_image.to_rgba8();
        let raw_data = rgba_image.as_raw();

        // Ensure data size is correct
        if raw_data.len() != self.frame_size {
            return Err(format!(
                "Frame size mismatch: expected {}, got {}",
                self.frame_size,
                raw_data.len()
            ).into());
        }
        // else{
        //     println!("Frame size match OK: expected {}, got {}", self.frame_size,raw_data.len());
        // }

        // Copy frame data into shared memory
        unsafe {
            let buffer = &mut self.shmem.as_slice_mut()[..self.frame_size];
            buffer.copy_from_slice(raw_data);
        }

        Ok(())
    }

    fn finish(self: Box<Self>) {
        // Shared memory will be cleaned up automatically
    }
}

unsafe impl Send for MyCustomEncoder {}
unsafe impl Sync for MyCustomEncoder {}
