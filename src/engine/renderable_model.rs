use core::num;
use std::iter::repeat;

use cgmath::Vector3;
use crate::graphics::Model;

use crate::graphics::Instance;

use super::{component_store::ComponentKey, transforms::ModelTransform, Scene};

#[derive(PartialEq, Clone, Copy, Debug)]
pub struct ModelDims {
  pub width: Option<f32>,
  pub height: Option<f32>,
  pub depth: Option<f32>
}

impl ModelDims {
  pub fn new(width: f32, height: f32, depth: f32) -> Self {
    Self {
      width: Some(width),
      height: Some(height),
      depth: Some(depth)
    }
  }

  pub fn empty() -> Self {
    Self {
      width: None,
      height: None,
      depth: None
    }
  }

  pub fn from_width(width: f32) -> Self {
    Self {
      width: Some(width),
      depth: None,
      height: None
    }
  }

  pub fn from_height(height: f32) -> Self {
    Self {
      height: Some(height),
      depth: None,
      width: None
    }
  }

  pub fn from_depth(depth: f32) -> Self {
    Self {
      depth: Some(depth),
      height: None,
      width: None
    }
  }

  pub fn set_width(&mut self, width: f32) {
    self.width = Some(width)
  }

  pub fn set_height(&mut self, height: f32) {
    self.height = Some(height)
  }

  pub fn set_depth(&mut self, depth: f32) {
    self.depth = Some(depth)
  }
}

#[derive(Clone, Debug, PartialEq)]
pub struct RenderSettings {
  pub instances: usize,
  pub opacities: Option<Vec<f32>>,
  pub dims: Option<Vec<ModelDims>>,
  pub transforms: Option<Vec<ModelTransform>>
}

#[derive(Clone, Debug, PartialEq)]
pub struct RenderInstance {
  pub transform: ModelTransform,
  pub opacity: f32,
  pub scale: Vector3<f32>
}

impl RenderSettings {
  pub fn default() -> RenderSettings {
    Self {
      instances: 1,
      opacities: None,
      dims: None,
      transforms: None,
    }
  }

  pub fn to_render_instances(&self, model: &Model) -> Vec<RenderInstance> {
    let mut out: Vec<RenderInstance> = Vec::new();
    let opacities = self.opacities.clone().unwrap_or(Vec::new());
    let dims = self.dims.clone().unwrap_or(Vec::new());
    let transforms = self.transforms.clone().unwrap_or(Vec::new());
    for i in 0..self.instances {
      let transform = transforms.get(i).unwrap_or(&ModelTransform::default()).clone();
      let model_size = model.bounds.map(|val| (val.1 - val.0).abs());
      let dims = dims.get(i).unwrap_or(&ModelDims::empty()).clone();
      let scale = Vector3::new(dims.width.unwrap_or(model_size[0])/ model_size[0], dims.height.unwrap_or(model_size[1])/ model_size[1], dims.depth.unwrap_or(model_size[2])/ model_size[2]);

      out.push(RenderInstance {
        transform,
        opacity: opacities.get(i).unwrap_or(&1.).clone(),
        scale
      })
    }
    out
  }
}

#[derive(PartialEq, Hash, Clone, Eq, Debug)]
pub struct RenderableModel {
  pub index: u32,
  component: ComponentKey,
  filename: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RenderableModelWithSettings(RenderableModel, RenderSettings);

impl RenderableModel {
  pub fn new(index: u32, key: ComponentKey, filename: &str) -> Self {
    Self {
      index,
      component: key,
      filename: filename.into()
    }
  }

  pub fn render(&self, scene: &mut Scene) -> Result<(), super::errors::EngineError> {
    let default_transform: ModelTransform = ModelTransform::default();
    scene.render_model(self, None)
  }

  pub fn instanced(&self, num_instances: u32) -> RenderableModelWithSettings {
    let render_settings = RenderSettings {
      instances: num_instances as usize,
      transforms: None,
      dims: None,
      opacities: None
    };
    RenderableModelWithSettings(self.clone(), render_settings)
  }

  pub fn transform(&self, transform: ModelTransform) -> RenderableModelWithSettings {
    let render_settings = RenderSettings {
      instances: 1,
      transforms: Some(Vec::from([transform])),
      dims: None,
      opacities: None
    };
    return RenderableModelWithSettings(self.clone(), render_settings)
  }

  pub fn opacity(&self, opacity: f32) -> RenderableModelWithSettings {
    let render_settings = RenderSettings {
      instances: 1,
      transforms: None,
      dims: None,
      opacities: Some(Vec::from([opacity]))
    };
    return RenderableModelWithSettings(self.clone(), render_settings)
  }

  pub fn dims(&self, dims: ModelDims) -> RenderableModelWithSettings {
    let render_settings: RenderSettings = RenderSettings {
      instances: 1,
      transforms: None,
      dims: Some(Vec::from([dims])),
      opacities: None
    };
    return RenderableModelWithSettings(self.clone(), render_settings)
  }

  pub fn width(&self, width: f32) -> RenderableModelWithSettings {
    let render_settings: RenderSettings = RenderSettings {
      instances: 1,
      transforms: None,
      dims: Some(Vec::from([ModelDims::from_width(width)])),
      opacities: None
    };
    return RenderableModelWithSettings(self.clone(), render_settings)
  }

  pub fn height(&self, height: f32) -> RenderableModelWithSettings {
    let render_settings: RenderSettings = RenderSettings {
      instances: 1,
      transforms: None,
      dims: Some(Vec::from([ModelDims::from_height(height)])),
      opacities: None
    };
    return RenderableModelWithSettings(self.clone(), render_settings)
  }

  pub fn depth(&self, depth: f32) -> RenderableModelWithSettings {
    let render_settings: RenderSettings = RenderSettings {
      instances: 1,
      transforms: None,
      dims: Some(Vec::from([ModelDims::from_depth(depth)])),
      opacities: None
    };
    return RenderableModelWithSettings(self.clone(), render_settings)
  }
}

impl RenderableModelWithSettings {
  pub fn render(&self, scene: &mut Scene) -> Result<(), super::errors::EngineError> {
    let transform = self.1.clone().transforms.unwrap_or(Vec::new()).get(0).unwrap_or(&ModelTransform::default()).clone(); 
    scene.render_model(&self.0, Some(self.1.clone()))
  }

  pub fn transform(&self, transform: ModelTransform) -> RenderableModelWithSettings {
    let transforms = std::iter::repeat(transform).take(self.1.instances).collect::<Vec<ModelTransform>>();
    let mut render_settings = self.1.clone();
    render_settings.transforms = Some(transforms);
    Self(self.0.clone(), render_settings)
  }

  pub fn opacity(&self, opacity: f32) -> RenderableModelWithSettings {
    let mut render_settings = self.1.clone();
    let opacities = repeat(opacity).take(self.1.instances).collect::<Vec<f32>>();
    render_settings.opacities = Some(opacities);
    Self(self.0.clone(), render_settings)
  }

  pub fn dims(&self, dims: ModelDims) -> RenderableModelWithSettings {
    let mut render_settings = self.1.clone();
    let dims_vec = repeat(dims).take(self.1.instances).collect::<Vec<ModelDims>>();
    render_settings.dims = Some(dims_vec);
    Self(self.0.clone(), render_settings)
  }

  pub fn width(&self, width: f32) -> RenderableModelWithSettings {
    let mut render_settings = self.1.clone();
    let mut dims = render_settings.dims.unwrap_or(repeat(ModelDims::from_width(width)).take(self.1.instances).collect());
    for dim in dims.iter_mut() { dim.set_width(width) }

    render_settings.dims = Some(dims);
    Self(self.0.clone(), render_settings)
  }

  pub fn height(&self, height: f32) -> RenderableModelWithSettings {
    let mut render_settings = self.1.clone();
    let mut dims = render_settings.dims.unwrap_or(repeat(ModelDims::from_height(height)).take(self.1.instances).collect());
    for dim in dims.iter_mut() { dim.set_height(height) }

    render_settings.dims = Some(dims);
    Self(self.0.clone(), render_settings)
  }

  pub fn depth(&self, depth: f32) -> RenderableModelWithSettings {
    let mut render_settings = self.1.clone();
    let mut dims = render_settings.dims.unwrap_or(repeat(ModelDims::from_depth(depth)).take(self.1.instances).collect());
    for dim in dims.iter_mut() { dim.set_depth(depth) }

    render_settings.dims = Some(dims);
    Self(self.0.clone(), render_settings)
  }

  pub fn transform_instanced(&self, transforms: Vec<ModelTransform>) -> RenderableModelWithSettings {
    let mut safe_transform_vec = transforms.clone();
    if safe_transform_vec.len() == 0 {
      let mut render_settings = self.1.clone();
      render_settings.transforms = None;
      return Self(self.0.clone(), render_settings)
    }
    size_to_fit(&mut safe_transform_vec, self.1.instances);

    let mut render_settings = self.1.clone();
    render_settings.transforms = Some(safe_transform_vec);
    Self(self.0.clone(), render_settings)
  }

  pub fn opacity_instanced(&self, opacities: Vec<f32>) -> RenderableModelWithSettings {
    let mut safe_opacity_vec = opacities.clone();
    if safe_opacity_vec.len() == 0 {
      let mut render_settings = self.1.clone();
      render_settings.opacities = None;
      return Self(self.0.clone(), render_settings)
    }
    size_to_fit(&mut safe_opacity_vec, self.1.instances);

    let mut render_settings = self.1.clone();
    render_settings.opacities = Some(safe_opacity_vec);
    Self(self.0.clone(), render_settings)
  }

  pub fn dims_instanced(&self, dims: Vec<ModelDims>) -> RenderableModelWithSettings {
    let mut safe_dims_vec = dims.clone();
    if safe_dims_vec.len() == 0 {
      let mut render_settings = self.1.clone();
      render_settings.opacities = None;
      return Self(self.0.clone(), render_settings)
    }
    size_to_fit(&mut safe_dims_vec, self.1.instances);

    let mut render_settings = self.1.clone();
    render_settings.dims = Some(safe_dims_vec);
    Self(self.0.clone(), render_settings)
  }
}

pub fn size_to_fit<T: Clone>(vec: &mut Vec<T>, target_len: usize) {
  if vec.len() == 0 {
    return;
  }

  if vec.len() < (target_len) {
    let last_elem = vec.last().unwrap().clone();
    let mut additional_elems = repeat(last_elem).take(target_len - vec.len()).collect::<Vec<T>>();
    vec.append(&mut additional_elems);
  } else if vec.len() > (target_len) {
    vec.truncate(target_len);
  }
}
