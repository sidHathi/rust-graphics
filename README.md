## Siddharth Hathi 2024

# Rust Graphics Engine

### Summary

This repo contains a component based graphics engine implemented using Rust and webgpu. The bulk of the engine is implemented in the [engine](src/engine). The Key classes are `component.rs` where I define the underlying data structure used to contain and render each graphical component and `scene.rs` which manages drawing and updating the wgpu scene. 

### Features of note:

* New components are defined by implementing the `ComponentFunctions` trait for an arbitrary struct. This involves implementing three methods: `init` - where the component is expected to initialize itself and its children in the scene, `update`, where the component receives updates from the scene, and `render` where the component asks the scene to draw any models or children that it likes
* The components render using both local and global coordinate systems (3D cartesian + quaternion). By default, the positioning and rotation of each component takes place relative to its parent. If the user would prefer to use global positioning, this is also supported. Since the positions of children and models relative to their parents are not known until render time, they are dynamically calculated and cached using the `transform_queue` data structure which keeps track of applied parent transforms during the render pass
* The engine supports shared global state. The user defines the state for their app in the `app_state.rs` function and `store.rs` implements a state store which supports retreival, updates, and adding listener callbacks for specific state changes. Any struct that implements the `ComponentFunctions` trait can listen for these state changes by implementing the `StateListener` trait
* The engine supports global event handling. Events are triggered through the scene's `EventManager` which triggers event listeners across all components listening for those specific events. Events include I/O events managed by the scene, and custom events created by the user.
* Once a struct implements the `ComponentFunctions` trait, it can be added to the scene by its parent by creating a new `Component` struct (initialized using an `Arc<Mutex<dyn ComponentFunctions>>`) using the `Component::new` function.
* The app support threaded execution of async code that mutates individual components
* The scene only renders one Component, which generates the scene using models and children

### Disclosure

This project is still very much in progress. I intend to add many additional features including dynamic lighting, animation, recurrent events, etc.. Documentation within the code itself is also nonexistent and will be added with time.