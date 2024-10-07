use core::f32;
use std::{collections::HashMap, hash::Hash, mem, sync::{Arc, Mutex, RwLock}};

use cgmath::{Matrix4, SquareMatrix, Vector3, Vector4};
use wgpu::util::DeviceExt;
use glyph_brush::{ab_glyph::FontArc, BrushError, BuiltInLineBreaker, Color, FontId, GlyphBrush, GlyphBrushBuilder, Rectangle, Section, Text};

use crate::{engine::{errors::EngineError, text::{text_vertex::TextModelData, VerticalTextAlignment}, transform_queue::TransformQueue, transforms::{ComponentTransform, GlobalTransform}}, graphics::{get_render_pipeline, Texture}};

use super::{cg_text::CGText, font::FontFace, text_vertex::TextVertex};

pub const FONT_DATA_ARRAYS: [(FontFace, &'static [u8]); 2] = [
  (FontFace::Default, include_bytes!("AtkinsonHyperlegible-Bold.ttf")),
  (FontFace::Atkinson, include_bytes!("AtkinsonHyperlegible-Bold.ttf")),
];

// pub struct TextBuffer {
//   pub num_vertices: u32,
//   pub vertex_buffer: wgpu::Buffer,
//   pub num_indices: u32,
//   pub index_buffer: wgpu::Buffer,
//   pub glyph_texture: Texture
// }

#[derive(Clone, Debug)]
pub struct TextRenderData {
  pub text: CGText,
  pub font_id: FontId,
  pub global_transform: GlobalTransform,
}

pub struct TextRenderer {
  pub font_map: HashMap<FontFace, FontId>,
  pub glyph_brush: Arc<Mutex<glyph_brush::GlyphBrush<()>>>,
  pub transform_queue: TransformQueue,
  pub text_render_pipeline: wgpu::RenderPipeline,
  pub text_model_buffer: wgpu::Buffer,
  pub text_vertex_buffer: wgpu::Buffer,
  pub num_text_vertices: RwLock<u32>,
  pub text_index_buffer: wgpu::Buffer,
  pub num_text_indices: RwLock<u32>,
  pub texture: Texture,
  pub texture_bind_group: wgpu::BindGroup,
  // pub text_transform_bind_group: wgpu::BindGroup,
  pub render_list: Vec<TextRenderData>
}

impl TextRenderer {
  pub fn new(
    device: &wgpu::Device, 
    queue: &wgpu::Queue,
    config: &wgpu::SurfaceConfiguration,
    surface_format: wgpu::TextureFormat,
    camera_bind_group_layout: &wgpu::BindGroupLayout,
    light_bind_group_layout: &wgpu::BindGroupLayout,
  ) -> Self {
    let mut font_map: HashMap<FontFace, FontId> = HashMap::new();
    let mut glyph_brush: Option<GlyphBrush<()>> = None;
    for (face, data) in FONT_DATA_ARRAYS {
      let font_res = FontArc::try_from_slice(data);
      if let Ok(font) = font_res {
        if glyph_brush.as_ref().is_none() {
          glyph_brush = Some(GlyphBrushBuilder::using_font(font).initial_cache_size((2048, 2048)).build());
          font_map.insert(face, FontId(0));
        } else {
          let id = glyph_brush.as_mut().unwrap().add_font(font);
          font_map.insert(face, id);
        }
      }
    }

    // initialize buffers
    let model_init = TextModelData { 
      model_matrix: Matrix4::identity().into(),
      color: [0., 0., 0.],
      opacity: 1. 
    };
    let text_model_buffer = device.create_buffer_init(
      &wgpu::util::BufferInitDescriptor {
        label: Some("Text model matrix"),
        contents: bytemuck::cast_slice(&[model_init]),
        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
      }
    );
    let text_vertex_buffer = device.create_buffer_init(
      &wgpu::util::BufferInitDescriptor {
        label: Some("Glyph vertex buffer"),
        contents: bytemuck::cast_slice(&[TextVertex::placeholder(); 2048] as &[TextVertex]),
        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST
      }
    );
    let text_index_buffer = device.create_buffer_init(
      &wgpu::util::BufferInitDescriptor {
        label: Some("Glyph index buffer"),
        contents: bytemuck::cast_slice(&[0 as u32; 2048] as &[u32]),
        usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST
      }
    );

    // Create a texture for the glyphs
    let glyph_texture_extent = wgpu::Extent3d {
      width: 2048,
      height: 2048,
      depth_or_array_layers: 1,
    };
    let glyph_texture = device.create_texture(&wgpu::TextureDescriptor {
      label: Some("Glyph Texture"),
      size: glyph_texture_extent,
      mip_level_count: 1,
      sample_count: 1,
      dimension: wgpu::TextureDimension::D2,
      format: wgpu::TextureFormat::R8Unorm,
      usage: wgpu::TextureUsages::TEXTURE_BINDING | 
      wgpu::TextureUsages::COPY_DST,
      view_formats: &[],
    });
    let glyph_texture_view = glyph_texture.create_view(&wgpu::TextureViewDescriptor::default());
    let glyph_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
      address_mode_u: wgpu::AddressMode::ClampToEdge,
      address_mode_v: wgpu::AddressMode::ClampToEdge,
      address_mode_w: wgpu::AddressMode::ClampToEdge,
      mag_filter: wgpu::FilterMode::Linear,
      min_filter: wgpu::FilterMode::Linear,
      mipmap_filter: wgpu::FilterMode::Linear,
      ..Default::default()
    });
    let texture = Texture {
      texture: glyph_texture,
      view: glyph_texture_view,
      sampler: glyph_sampler
    };
    
    // texture bind group
    let texture_bind_group_layout = device.create_bind_group_layout(
      &wgpu::BindGroupLayoutDescriptor { 
        label: Some("Glyph texture bind group layout"), 
        entries: &[
          wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Texture {
              multisampled: false,
              view_dimension: wgpu::TextureViewDimension::D2,
              sample_type: wgpu::TextureSampleType::Float { filterable: true },
            },
            count: None,
          },
          wgpu::BindGroupLayoutEntry {
            binding: 1,
            visibility: wgpu::ShaderStages::FRAGMENT,
            // This should match the filterable field of the
            // corresponding Texture entry above.
            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
            count: None,
          },
        ] 
      }
    );
    let texture_bind_group = device.create_bind_group(
      &wgpu::BindGroupDescriptor {
        layout: &texture_bind_group_layout,
        label: Some("text texture bind group"),
        entries: &[
          wgpu::BindGroupEntry {
            binding: 0,
            resource: wgpu::BindingResource::TextureView(&texture.view),
          },
          wgpu::BindGroupEntry {
            binding: 1,
            resource: wgpu::BindingResource::Sampler(&texture.sampler),
          },
        ]
      }
    );

    // render pipeline
    let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
      label: Some("Text Render Pipeline Layout"),
      bind_group_layouts: &[
        &texture_bind_group_layout,
        camera_bind_group_layout,
        light_bind_group_layout,
        // &text_transform_bind_layout
      ],
      push_constant_ranges: &[],
    });

    use crate::graphics::Vertex;
    let text_render_pipeline = {
      let shader = wgpu::ShaderModuleDescriptor {
          label: Some("Text Shader"),
          source: wgpu::ShaderSource::Wgsl(include_str!("text_shader.wgsl").into()),
      };
      get_render_pipeline(
        &device,
        &render_pipeline_layout,
        config.format,
        Some(Texture::DEPTH_FORMAT),
        &[TextVertex::desc(), TextModelData::desc()],
        shader,
        "vs_main", 
        "fs_main"
      )
    };

    Self {
      font_map,
      glyph_brush: Arc::new(Mutex::new(glyph_brush.unwrap())),
      transform_queue: TransformQueue::new(),
      text_render_pipeline,
      render_list: Vec::new(),
      text_model_buffer,
      text_index_buffer,
      num_text_indices: RwLock::new(0),
      text_vertex_buffer,
      num_text_vertices: RwLock::new(0),
      texture,
      texture_bind_group,
      // text_transform_bind_group
    }
  }

  // returns (num vertices loaded, num indices loaded)
  fn load_text_into_buffers(
    &self,
    text: TextRenderData, 
    queue: &wgpu::Queue,
    config: &wgpu::SurfaceConfiguration
  ) -> Result<(u32, u32), BrushError> {
    let section = text.text.get_glyph_section();

    let scaling_mat = Matrix4::from_scale(0.02);
    let model_matrix: [[f32; 4]; 4] = (text.global_transform.as_matrix() * scaling_mat).into();
    let text_color = text.text.color;
    queue.write_buffer(&self.text_model_buffer, 0, bytemuck::cast_slice(&[TextModelData { 
      model_matrix,
      color: [text_color.r as f32, text_color.g as f32, text_color.b as f32],
      opacity: text.text.opacity
     }]));

    let text_vertices: Mutex<Vec<TextVertex>> = Mutex::new(vec![]);
    let text_indices: Mutex<Vec<u32>> = Mutex::new(vec![]);
    let index_offset = Mutex::new(0);
    let max_y_pos: RwLock<f32> = RwLock::new(f32::NEG_INFINITY);
    let min_y_pos: RwLock<f32> = RwLock::new(f32::INFINITY);

    self.glyph_brush.lock().unwrap().queue(&section);
    let res = self.glyph_brush.lock().unwrap().process_queued(
      |rect, tex_data| {
        if tex_data.len() > 0 {
          println!("{:?}", tex_data);
          queue.write_texture(
            wgpu::ImageCopyTexture {
              texture: &self.texture.texture,
              mip_level: 0,
              origin: wgpu::Origin3d {
                x: rect.min[0] as u32,
                y: rect.min[1] as u32,
                z: 0,
              },
              aspect: wgpu::TextureAspect::All,
            },
            tex_data,
            wgpu::ImageDataLayout {
              offset: 0,
              bytes_per_row: Some(rect.width() as u32),
              rows_per_image: None,
            },
            wgpu::Extent3d {
              width: rect.width() as u32,
              height: rect.height() as u32,
              depth_or_array_layers: 1,
            },
          );
        }
      },
      |vertex_data| {
        println!("Y coords for text glyph {}, {}", vertex_data.pixel_coords.max.y, vertex_data.pixel_coords.min.y);
        if vertex_data.pixel_coords.max.y > max_y_pos.read().unwrap().clone() {
          *max_y_pos.write().unwrap() = vertex_data.pixel_coords.max.y;
        } else if vertex_data.pixel_coords.max.y < min_y_pos.read().unwrap().clone() {
          *min_y_pos.write().unwrap() = vertex_data.pixel_coords.max.y;
        }
        if vertex_data.pixel_coords.min.y > max_y_pos.read().unwrap().clone() {
          *max_y_pos.write().unwrap() = vertex_data.pixel_coords.min.y;
        } else if vertex_data.pixel_coords.min.y < min_y_pos.read().unwrap().clone() {
          *min_y_pos.write().unwrap() = vertex_data.pixel_coords.min.y;
        }
        let y_offset = vertex_data.pixel_coords.max.y - vertex_data.pixel_coords.min.y;
        let minX_scaled = vertex_data.pixel_coords.min.x;
        let maxX_scaled = vertex_data.pixel_coords.max.x;
        let minY_scaled = vertex_data.pixel_coords.min.y;
        let maxY_scaled = vertex_data.pixel_coords.max.y;
        let quad_vertices = [
            TextVertex {
                position: [minX_scaled, minY_scaled, 0.],
                tex_coords: [vertex_data.tex_coords.min.x, vertex_data.tex_coords.min.y],
            },
            TextVertex {
                position: [maxX_scaled, minY_scaled, 0.],
                tex_coords: [vertex_data.tex_coords.max.x, vertex_data.tex_coords.min.y],
            },
            TextVertex {
                position: [maxX_scaled, maxY_scaled, 0.],
                tex_coords: [vertex_data.tex_coords.max.x, vertex_data.tex_coords.max.y],
            },
            TextVertex {
                position: [minX_scaled, maxY_scaled, 0.],
                tex_coords: [vertex_data.tex_coords.min.x, vertex_data.tex_coords.max.y],
            },
        ];
        for vert in quad_vertices {
          text_vertices.lock().unwrap().push(vert);
        }
        let curr_offset = index_offset.lock().unwrap().clone();
        text_indices.lock().unwrap().push(curr_offset);
        text_indices.lock().unwrap().push(curr_offset + 2);
        text_indices.lock().unwrap().push(curr_offset + 1);
        text_indices.lock().unwrap().push(curr_offset);
        text_indices.lock().unwrap().push(curr_offset + 3);
        text_indices.lock().unwrap().push(curr_offset + 2);

        // Update the index offset for the next glyph
        *index_offset.lock().unwrap() += 4;
      }
    );
    if res.is_err() {
      return Err(res.err().unwrap());
    }

    let mut text_vertex_vec = text_vertices.lock().unwrap().clone();
    let text_index_vec = text_indices.lock().unwrap().clone();

    let num_text_indices = text_index_vec.len() as u32;
    let num_text_vertices = text_vertex_vec.len() as u32;

    if num_text_indices == 0 || num_text_vertices == 0 {
      return Ok((self.num_text_vertices.read().unwrap().clone(), self.num_text_indices.read().unwrap().clone()))
    }

    let max_y_unwrapped = max_y_pos.read().unwrap().clone();
    let min_y_unwrapped = min_y_pos.read().unwrap().clone();
    println!("Text index vec: {:?} contains {} indices", text_index_vec, num_text_indices);
    println!("Text vertex vec: {:?} contains {} vertices", text_vertex_vec, num_text_vertices);
    println!("max y pos for text: {}", max_y_unwrapped);
    for vert in text_vertex_vec.iter_mut() {
      println!("Vert position: {:?}", vert.position);
      if text.text.vertical_text_alignment == VerticalTextAlignment::Top || text.text.vertical_text_alignment == VerticalTextAlignment::Center {
        vert.position[1] = max_y_unwrapped - vert.position[1];
      } else {
        vert.position[1] = min_y_unwrapped - vert.position[1];
      }
      println!("Updating vert position: {:?}", vert.position);
    }

    queue.write_buffer(&self.text_vertex_buffer, 0, bytemuck::cast_slice(&text_vertex_vec));
    queue.write_buffer(&self.text_index_buffer, 0, bytemuck::cast_slice(&text_index_vec));

    *self.num_text_vertices.write().unwrap() = num_text_vertices;
    *self.num_text_indices.write().unwrap() = num_text_indices;
    return Ok((num_text_vertices, num_text_indices))
  }

  pub fn start_component_render(&mut self, transform: ComponentTransform) {
    self.transform_queue.push(transform)
  }

  pub fn end_component_render(&mut self) -> Option<ComponentTransform> {
    self.transform_queue.pop()
  }

  pub fn render_text(&mut self, text: CGText) {
    let global_transform = self.transform_queue.transform_mt(&text.transform);
    let font_id = self.font_map.get(&text.font_face).unwrap_or(&FontId(0));
    let render_data: TextRenderData = TextRenderData {
      global_transform,
      font_id: font_id.clone(),
      text
    };

    // println!("Rendering text: {:?}", render_data);
    self.render_list.push(render_data);
  }

  pub fn reset(&mut self) {
    self.render_list.clear();
  }
}

pub trait DrawText<'a> {
  fn draw_text(
    &mut self,
    text_renderer: &'a TextRenderer,
    device: &'a wgpu::Device, 
    queue: &'a wgpu::Queue,
    config: &'a wgpu::SurfaceConfiguration,
    camera_bind_group: &'a wgpu::BindGroup,
    light_bind_group: &'a wgpu::BindGroup
  );
}

impl<'a, 'b> DrawText<'b> for wgpu::RenderPass<'a> where 'b: 'a {
  fn draw_text(
    &mut self,
    text_renderer: &'a TextRenderer,
    device: &'a wgpu::Device, 
    queue: &'a wgpu::Queue,
    config: &'a wgpu::SurfaceConfiguration,
    camera_bind_group: &'a wgpu::BindGroup,
    light_bind_group: &'a wgpu::BindGroup
  ) {
    self.set_pipeline(&text_renderer.text_render_pipeline);
    for text in text_renderer.render_list.iter() {
      // load text info into buffers
      if let Ok((num_vertices, num_indices)) = text_renderer.load_text_into_buffers(text.clone(), queue, config) {
        // println!("Text buffer loaded successfuly with {} indices, {} vertices", num_indices, num_vertices);
        // draw the buffers
        self.set_vertex_buffer(1, text_renderer.text_model_buffer.slice(..));
        self.set_vertex_buffer(0, text_renderer.text_vertex_buffer.slice(..));
        self.set_index_buffer(text_renderer.text_index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        self.set_bind_group(0, &text_renderer.texture_bind_group, &[]);
        self.set_bind_group(1, camera_bind_group, &[]);
        self.set_bind_group(2, light_bind_group, &[]);
        // self.set_bind_group(3, &text_renderer.text_transform_bind_group, &[]);
        self.draw_indexed(0..num_indices, 0, 0..1);
      }
    }
  }
}