use tokio::runtime::Runtime;

use crate::{graphics::Texture, util::log_buffer_data};

pub fn readback_shadows(
  shadow_dt: &Texture,
  device: &wgpu::Device,
  queue: &wgpu::Queue,
  config: &wgpu::SurfaceConfiguration
) {
  let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
    label: Some("Copy buffer encoder")
  });
  let buffer_size = (config.width * config.height * std::mem::size_of::<f32>() as u32) as u64;

  // Create a buffer to copy the texture data to
  let readback_buffer = device.create_buffer(&wgpu::BufferDescriptor {
      label: Some("Readback Buffer"),
      size: buffer_size,
      usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
      mapped_at_creation: false,
  });

  // Specify the texture copy operation
  encoder.copy_texture_to_buffer(
    wgpu::ImageCopyTexture {
      texture: &shadow_dt.texture,
      mip_level: 0,
      origin: wgpu::Origin3d::ZERO,
      aspect: wgpu::TextureAspect::All,
    },
    wgpu::ImageCopyBuffer {
      buffer: &readback_buffer,
      layout: wgpu::ImageDataLayout {
        offset: 0,
        bytes_per_row: Some(config.width * std::mem::size_of::<f32>() as u32),
        rows_per_image: Some(config.height as u32),
      },
    },
    wgpu::Extent3d {
      width: config.width,
      height: config.height,
      depth_or_array_layers: 1,
    },
  );

  queue.submit(std::iter::once(encoder.finish()));

  let buffer_slice: wgpu::BufferSlice = readback_buffer.slice(..);
  let (tx, rx) = futures::channel::oneshot::channel();

  buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
    let _ = tx.send(result);
  });

  device.poll(wgpu::Maintain::Wait);

  let rt = Runtime::new().unwrap();
  rt.block_on(async {
    rx
      .await
      .expect("communication failed")
      .expect("buffer reading failed");
    let slice: &[u8] = &buffer_slice.get_mapped_range();
    let mut num_shadowed: u32 = 0;
    let mut num_lit: u32 = 0;
    for (idx, val) in slice.iter().enumerate() {
      if *val != 0 {
        num_lit += 1;
        // println!("Val {} found at row {}, col {}", *val, idx/(config.width as usize), idx - (config.width as usize * (idx/(config.width as usize))));
      } else {
        num_shadowed += 1;
        // println!("Val {} found at row {}, col {}", *val, idx/(config.width as usize), idx - (config.width as usize * (idx/(config.width as usize))));
      }
    }
    // println!("Percentage of shadowed area: {} -> num lit pixels: {}, num shadowed: {}", (num_shadowed as f32)/((num_lit + num_shadowed) as f32), num_lit, num_shadowed);
    let _ = log_buffer_data(slice, config, "shadow_map_dump.log");
  });
}