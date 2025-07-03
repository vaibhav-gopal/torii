# Game Engine Blueprint

## 1. CORE — Must-have Parts

### A. ECS

* Use a minimal ECS (e.g., `hecs` or your own sparse set).
* Keep your `World` and `Component` storage decoupled from rendering, physics, input.
* Components should be plain data.
* Systems are functions that run on ECS queries.
* Schedule runs your systems in order.

### B. Staged Scheduler

* Define stages (`Input`, `PreUpdate`, `Update`, `PostUpdate`, `Render`).
* Let systems be registered to a specific stage for stable hooks.
* `EngineBuilder` accumulates systems/resources.
* `Engine` runs the schedule in order.

### C. `wgpu` Render Backend

* Use `wgpu` for cross-platform graphics (Vulkan/Metal/DX/WebGPU).
* Use `winit` for window creation and input handling.
* The RenderSystem queries the ECS for `Position` and `Renderable`.
* Upload instance buffers and issue draw calls.
* Keep GPU resources (`Pipeline`, `Buffer`, `Texture`) behind your own `Renderer` trait so you can swap `wgpu` for `ash` later.

### D. Plugin System

* Basic `Plugin` trait:

  ```rust
  trait Plugin {
      fn build(&self, engine: &mut EngineBuilder);
  }
  ```

* Plugins can register systems, add resources, and hook into your defined stages.

* Plugins can depend on other plugins if needed.

### E. Tracing & Logging

* Use `tracing` and `tracing-subscriber` for structured logs and spans.
* Wrap this in an `EngineLogger`:

    * Buffer logs for an in-game debug console if needed.
    * Route logs to file, terminal, or console.
    * Use spans for profiling phases (`ecs_update`, `render_pass`).

## 2. OPTIONAL

### Reflection

* Skip for version 1 unless you want a custom inspector/editor.
* If needed later, look at Bevy’s `Reflect` pattern or write your own derive macro.

### Physics / Audio

* Keep physics and audio as plugin hooks for now.
* Example plugins: `PhysicsPlugin`, `AudioPlugin`.
* Stub these out until you’re ready to implement them.

### WASM Plugins

* Skip for version 1.
* Keep the `Plugin` system modular so you could add a WASM runtime in the future.
* If you do add this later, expose a stable ABI for ECS operations.

### Serialization / Save Files

* Use `serde` for saving/loading if needed.
* Keep component data `Serialize` and `Deserialize` compatible.
* Skip complex hot reload logic for now.

## 3. FUTURE ME

* Raw Vulkan backend (`ash`) when you want mesh shading, bindless, or ray tracing.
* Custom profiling/telemetry: tie `tracing` spans to Tracy or export flamegraphs.
* In-game editor: reflection plus GUI to inspect ECS world, save/load scenes.
* Headless server mode: run ECS and netcode with no window/render backend.
* Asset pipeline: hot reload assets, versioned asset manifest.

## 4. Minimum Viable Engine Summary

* ECS + Schedule + Plugin hooks.
* `wgpu` backend behind a `Renderer` trait.
* `winit` for window/input.
* `tracing` for logs and spans.
* Staged plugin system for clear extension points.
* No magic, reflection, or WASM yet — leave those for future expansion.