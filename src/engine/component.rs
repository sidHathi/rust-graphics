use std::{any::Any, future::Future, ops::Deref, rc::Rc, sync::{Arc, Mutex, MutexGuard}};

use cgmath::Point3;
use tokio::runtime::Runtime;

use crate::graphics::{DrawModel, Model};

use super::{component_store::ComponentKey, errors::EngineError, events::{Event, EventKey, EventListener}, model_renderer::ModelRenderer, state::StateListener, transforms::ComponentTransform, Scene};
use async_trait::async_trait;

#[async_trait(?Send)]
pub trait ComponentFunctions: Any + Send + Sync + EventListener + StateListener {
  // initialize the component
  async fn init(
    &mut self,
    scene: &mut Scene,
    key: ComponentKey,
    parent: Option<ComponentKey>,
  );

  // update is called every frame
  fn update(&mut self, scene: &mut Scene, dt: instant::Duration) {
    return;
  }

  // get models to be rendered when this component is rendered
  fn render(&self, scene: &mut Scene) -> Result<(), EngineError> {
    Ok(())
  }
}

pub trait AsyncCallbackHandler<T>: ComponentFunctions + Any {
  fn handle_async_res(&mut self, data: T) -> ();
}

// Sized wrapper for a ComponentFunctions implementing struct
// Allows multiple mutable access and storage within the scene
#[derive(Clone)]
pub struct Component {
  pub key: ComponentKey, // key used to access the component in the scenes Component map
  underlying: Arc<Mutex<dyn ComponentFunctions>>,
}

impl Component {
  // Takes in an arc mutex pointer to a ComponentFunctions implementing struct,
  // creates a new Component wrapper for the struct, initializes it, and adds
  // it to the scene
  pub async fn new<T: ComponentFunctions>(
    underlying: Arc<Mutex<T>>,
    scene: &mut Scene,
    parent: Option<ComponentKey>
  ) -> Option<Component> {
    let mut component = Self {
      key: ComponentKey::zero(),
      underlying: underlying as Arc<Mutex<dyn ComponentFunctions + 'static>>
    };
    let key_res = scene.components.insert(component.clone());
    if let Ok(key) = key_res {
      component.key = key;
      component.clone().init(scene, key.clone(), parent).await;
      return Some(component);
    }
    None
  }

  // initialize the underlying component
  pub async fn init(
    &mut self,
    scene: &mut Scene,
    key: ComponentKey,
    parent: Option<ComponentKey>,
  ) {
    self.underlying.lock().unwrap().init(scene, key, parent).await;
  }

  // update the underlying component
  pub fn update(&self, scene: &mut Scene, dt: instant::Duration) {
    self.underlying.lock().unwrap().update(scene, dt);
  }

  // render the component
  pub fn render(&self, scene: &mut Scene, transform: Option<ComponentTransform>) -> Result<(), EngineError> {
    scene.model_renderer.start_component_render(transform, self.key);
    let res = self.underlying.lock().unwrap().render(scene);
    scene.model_renderer.end_component_render();
    res
  }

  // used to execute async code which requires mutable access to a component
  // outside of the component itself (this is an unsafe operation)
  pub fn exec_async_unsafe<Args, Out, F, Fut>(underlying: Arc<Mutex<Box<dyn ComponentFunctions>>>, func: F, args: Args)
  where
    F: FnOnce(Arc<Mutex<Box<dyn AsyncCallbackHandler<Out>>>>, Args) -> Fut + Send + 'static,
    Fut: Future<Output = Out> + Send + 'static,
    Args: Send + Sync + 'static,
    Out: Send + Sync + 'static {
    let raw = Arc::into_raw(underlying) as *const Mutex<Box<dyn AsyncCallbackHandler<Out>>>;
    let unsafe_casted: Arc<Mutex<Box<dyn AsyncCallbackHandler<Out>>>> = unsafe { Arc::from_raw(raw) };

    // in new thread:
    let comp_mutex = unsafe_casted.clone();
    std::thread::spawn(move || {
      let rt = Runtime::new().unwrap();
      let out = rt.block_on(async {
        (func)(unsafe_casted, args).await
      });
      comp_mutex.lock().unwrap().handle_async_res(out);
    });
  }

  // used to execute async code that mutates a component within the component itself
  pub fn exec_async<CType: AsyncCallbackHandler<Out>, Args, Out, F, Fut>(underlying: Arc<Mutex<Box<CType>>>, func: F, args: Args)
  where
    F: FnOnce(Arc<Mutex<Box<CType>>>, Args) -> Fut + Send + 'static,
    Fut: Future<Output = Out> + Send + 'static,
    Args: Send + Sync + 'static,
    Out: Send + Sync + 'static
  {
    // in new thread
    let comp_mutex = underlying.clone();
    std::thread::spawn(move || {
      let rt = Runtime::new().unwrap();
      let out = rt.block_on(async {
        (func)(underlying, args).await
      });
      comp_mutex.lock().unwrap().handle_async_res(out);
    });
  }
}

// event listener and state listener are delegated to underlying
impl EventListener for Component {
  fn handle_event(&mut self, event: super::events::Event) {
    // Check if the trait object also implements AnotherTrait
    self.underlying.lock().unwrap().handle_event(event)
  }
}

impl StateListener for Component {
  fn handle_state_change(&mut self, key: String, state: &super::state::State) {
      self.underlying.lock().unwrap().handle_state_change(key, state)
  }
}