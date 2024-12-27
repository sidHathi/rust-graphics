use std::{sync::{Arc, Mutex}};

use async_trait::async_trait;
use cgmath::{EuclideanSpace, Point3, Quaternion};

use crate::engine::{component::ComponentFunctions, component_store::ComponentKey, events::EventListener, renderable_model::{ModelDims, RenderableModel}, state::StateListener, transforms::ModelTransform, Scene};

 const NET_DIM: u32 = 10;
 const NET_SPACING: f32 = 20.;

pub struct DebugSpheres {
  key: ComponentKey,
  parent: Option<ComponentKey>,
  model: Option<RenderableModel>,
  active: bool,
  mem: Option<Arc<Mutex<Self>>>,
}

#[async_trait(?Send)]
impl ComponentFunctions for DebugSpheres {
  async fn init(
    &mut self,
    scene: &mut Scene,
    key: ComponentKey,
    parent: Option<ComponentKey>
  ) {
    self.key = key;
    self.active = true;
    self.parent = parent;
    if let Ok(model) = scene.load_model("dice.obj", None, key).await {
      self.model = Some(model);
    } else {
      self.model = None;
    }
  }

  fn update(&mut self, _: &mut crate::engine::Scene, _:instant::Duration) {
    return;
  }

  fn render(&self,scene: &mut Scene) -> Result<(),crate::engine::errors::EngineError> {
    let mut positions: Vec<Point3<f32>> = Vec::new();
    // need to generate a network of positions and rotations
    let half_offset = NET_SPACING * NET_DIM as f32 / 2.;
    for row in 0..NET_DIM {
      for col in 0..NET_DIM {
        let x = row as f32 * NET_SPACING - half_offset;
        let y = col as f32 * NET_SPACING - half_offset;
        positions.push([x, y, 0.].into());
      }
    }

    for pos in positions {
      if let Some(model) = &self.model {
        println!("rendering model {:?}", model);
        let _ = model
          .transform(ModelTransform::local(pos.to_vec(), Quaternion::new(1., 0., 0., 0.)))
          .dims(ModelDims::new(5., 5., 5.))
          .opacity(1.)
          .render(scene);
      }
    }
    Ok(())
  }
}

impl EventListener for DebugSpheres {

}

impl StateListener for DebugSpheres {

}

impl DebugSpheres {
  pub fn new() -> Arc<Mutex<Self>> {
    let new_self = Self {
      key: ComponentKey::zero(),
      parent: None,
      active: false,
      mem: None,
      model: None,
    };
    let mem = Arc::new(Mutex::new(new_self));
    mem.lock().unwrap().mem = Some(mem.clone());
    mem
  }
}