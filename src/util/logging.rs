use std::{fs, io::{self, Write}};

pub fn log_buffer_data(
  data: &[u8], 
  config: &wgpu::SurfaceConfiguration,
  file_path: &str,
) -> Result<(), io::Error> {
  let header_str = format!("screen_width: {}, screen_height: {}\n", config.width, config.height);
  let file_res = fs::OpenOptions::new()
    .create(true)
    .write(true)
    .open(file_path);
  if let Ok(file) = file_res {
    let mut writer = io::BufWriter::new(file);
    writer.write_all(header_str.as_bytes())?;
    writer.write_all(data)?;
    writer.flush()?;
    return Ok(())
  }
  Err(file_res.unwrap_err())
}