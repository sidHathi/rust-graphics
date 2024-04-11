use image::GenericImageView;
use anyhow::*;

pub struct Texture {
  pub texture: wgpu::Texture,
  pub view: wgpu::TextureView,
  pub sampler: wgpu::Sampler,
}

impl Texture {
  pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

  pub fn create_depth_texture(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration, label: &str) -> Self {
    let size = wgpu::Extent3d {
      width: config.width,
      height: config.height,
      depth_or_array_layers: 1
    };
    let texture = device.create_texture(
      &wgpu::TextureDescriptor {
        label: Some(label),
        size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: Self::DEPTH_FORMAT,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT // 3.
            | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
      }
    );

    let view = texture.create_view(
      &wgpu::TextureViewDescriptor::default()
    );
    let sampler = device.create_sampler(
      &wgpu::SamplerDescriptor { 
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        mipmap_filter: wgpu::FilterMode::Nearest,
        compare: Some(wgpu::CompareFunction::LessEqual), // 5.
        lod_min_clamp: 0.0,
        lod_max_clamp: 100.0,
        ..Default::default()
      }
    );

    Self {
      texture,
      view,
      sampler,
    }
  }

  pub fn from_bytes(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    bytes: &[u8],
    label: &str,
    is_normal_map: bool,
  ) -> Result<Self> {
    let img = image::load_from_memory(bytes)?;
    Self::from_image(device, queue, &img, Some(label), is_normal_map)
  }

  pub fn from_image(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    img: &image::DynamicImage,
    label: Option<&str>,
    is_normal_map: bool,
  ) -> Result<Self> {
    let format = if is_normal_map {
      wgpu::TextureFormat::Rgba8Unorm
    } else {
      wgpu::TextureFormat::Rgba8UnormSrgb
    };

    let rgba = img.to_rgba8();
    let dimensions = img.dimensions();

    let texture_size = wgpu::Extent3d {
      width: dimensions.0,
      height: dimensions.1,
      depth_or_array_layers: 1
    };
    let texture = device.create_texture(
      &wgpu::TextureDescriptor {
        // All textures are stored as 3D, we represent our 2D texture
        // by setting depth to 1.
        size: texture_size,
        mip_level_count: 1, // We'll talk about this a little later
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        // Most images are stored using sRGB, so we need to reflect that here.
        format,
        // TEXTURE_BINDING tells wgpu that we want to use this texture in shaders
        // COPY_DST means that we want to copy data to this texture
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        label: Some("diffuse_texture"),
        // This is the same as with the SurfaceConfig. It
        // specifies what texture formats can be used to
        // create TextureViews for this texture. The base
        // texture format (Rgba8UnormSrgb in this case) is
        // always supported. Note that using a different
        // texture format is not supported on the WebGL2
        // backend.
        view_formats: &[],
      }
    );
    queue.write_texture(
      wgpu::ImageCopyTexture {
        texture: &texture,
        mip_level: 0,
        origin: wgpu::Origin3d::ZERO,
        aspect: wgpu::TextureAspect::All,
      }, 
      &rgba, 
      wgpu::ImageDataLayout {
        offset: 0,
        bytes_per_row: Some(4 * dimensions.0),
        rows_per_image: Some(dimensions.1),
      },
      texture_size
    );

    let view = texture.create_view(
      &wgpu::TextureViewDescriptor::default()
    );
    let sampler = device.create_sampler(
      &wgpu::SamplerDescriptor {
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Nearest,
        mipmap_filter: wgpu::FilterMode::Nearest,
        ..Default::default()
      }
    );

    Ok(Self {
      texture,
      view,
      sampler,
    })
  }
}
