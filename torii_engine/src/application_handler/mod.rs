use std::ops::Deref;
use winit::window::{Window, WindowAttributes, WindowId};
use winit::application::ApplicationHandler;
use winit::dpi::{LogicalPosition, LogicalSize};
use winit::event_loop::{ActiveEventLoop, EventLoop, EventLoopProxy};
use winit::event::{WindowEvent};

mod error;
pub use error::*;

pub enum AppEvents {
    CreateWindow(Option<WindowAttributes>),
    KillWindow(WindowId)
}

pub struct WindowAttributeBuilder {
    inner: WindowAttributes,
    callback: Option<Box<dyn FnMut(WindowAttributes) -> WindowAttributes>>
}

impl Default for WindowAttributeBuilder {
    fn default() -> Self {
        WindowAttributeBuilder {
            inner: WindowAttributes::default()
                .with_title("Torii Application")
                .with_inner_size(LogicalSize::new(800, 600))
                .with_position(LogicalPosition::new(0, 0)),
            callback: None
        }
    }
}

impl WindowAttributeBuilder {
    pub fn new(inner: WindowAttributes, callback: Option<Box<dyn FnMut(WindowAttributes) -> WindowAttributes>>) -> Self {
        WindowAttributeBuilder {
            inner,
            callback,
        }
    }
    
    pub fn build(&self) -> WindowAttributes {
        let mut attr = self.inner.clone();
        if let Some(&callback) = self.callback.as_ref() {
            attr = (*callback)(attr)
        }
        attr
    }

    pub fn set_inner(&mut self, inner: WindowAttributes) {
        self.inner = inner;
    }

    pub fn set_callback(&mut self, callback: Option<Box<dyn FnMut(WindowAttributes) -> WindowAttributes>>) {
        self.callback = callback;
    }
}

enum WindowState {
    Active(Window),
    Suspended
}

struct WindowObject {
    state: WindowState,
    attr: WindowAttributes,
    uuid: u32
}

struct AppHandler {
    event_loop: Option<EventLoop<AppEvents>>,
    event_loop_proxy: EventLoopProxy<AppEvents>,
    error_callback: Option<Box<dyn FnMut(Error)>>,
    windows: Vec<WindowObject>,
    attr_builder: WindowAttributeBuilder,
}

impl AppHandler {
    // PUBLIC FUNCTIONS (chainable, state changing functions)
    pub fn new() -> Result<Self> {
        let event_loop: EventLoop<AppEvents> = EventLoop::<AppEvents>::with_user_event()
            .build()
            .map_err(|e| InitializationError::EventLoopCreationError(e))?;
        event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);

        let event_loop_proxy = event_loop.create_proxy();

        let app_handler = AppHandler {
            event_loop: Some(event_loop),
            event_loop_proxy,
            error_callback: None,
            windows: vec![],
            attr_builder: WindowAttributeBuilder::default()
        };

        Ok(app_handler)
    }
    pub fn start_loop(mut self) -> Result<Self> {
        self.event_loop.take()
            .ok_or(StartLoopError::EventLoopAlreadyConsumedError)?
            .run_app(&mut self)
            .map_err(|e| StartLoopError::EventLoopRunAppError(e))?;
        Ok(self)
    }
    
    // SETTERS, GETTERS, CALLBACK EXECUTORS (non chainable)
    pub fn windows(&self) -> &Vec<WindowObject> {
        &self.windows
    }
    pub fn set_error_callback(&mut self, error_callback: Option<Box<dyn FnMut(Error)>>) {
        self.error_callback = error_callback;
    }
    pub fn error_callback(&mut self, error: Error) {
        match self.error_callback.as_mut() {
            Some(callback) => {(*callback)(error)}
            None => {}
        }
    }
    pub fn send_event(&mut self, event: AppEvents) -> Result<()>{
        self.event_loop_proxy.send_event(event)
            .map_err(|_| EventLoopProxyError::EventLoopProxySendEventError)?;
        Ok(())
    }
}

impl AppHandler {
    // PRIVATE FUNCTIONS (called from the event loop ; no return value)
    fn resume_all(&mut self, event_loop: &ActiveEventLoop) {
        if (!self.resumed) {
            for window_wrapper in self.windows.iter_mut() {
                let window_result = event_loop
                    .create_window(window_wrapper.attr.clone())
                    .map_err(|e| WindowCreationError::OSWindowCreationError(e));
                
                match window_result {
                    Ok(window) => {
                        window_wrapper.inner = Some(window);
                    }
                    Err(error) => {
                        self.error_callback(error.into());
                    }
                }
            }
        }
        self.resumed = true;
    }
    
    fn suspend_all(&mut self, event_loop: &ActiveEventLoop) {
        if (self.resumed) {
            for window in self.windows.iter_mut() {
                drop(window.inner.take()); // take owned window and drop it
            }
        }
        self.resumed = false;
    }
    
    fn create_window(&mut self, event_loop: &ActiveEventLoop, attributes: Option<WindowAttributes>) {
        let attributes = attributes.unwrap_or(self.attr_builder.build());
        
        let window_result = event_loop
            .create_window(attributes.clone())
            .map_err(|e| WindowCreationError::OSWindowCreationError(e));

        match window_result {
            Ok(window) => {
                self.windows.push(
                    WindowObject {
                        state: WindowState::Active(window),
                        attr: attributes,
                    }
                );
            },
            Err(error) => {
                self.error_callback(error.into());
            }
        };
    }
    
    fn destroy_window(&mut self, event_loop: &ActiveEventLoop, id: WindowId) {
        let window_index = match self.windows.iter()
            .position(|obj| if let WindowState::Active(window) = &obj.state {window.id() == id} else {false})
            .ok_or(WindowAccessError::WindowNotFoundError(id.into())) {
            Ok(idx) => idx,
            Err(error) => {
                self.error_callback(error.into());
                return;
            },
        };
        
        self.windows.swap_remove(window_index);
    }
}

impl ApplicationHandler<AppEvents> for AppHandler {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.resume_all(event_loop);
    }

    fn suspended(&mut self, event_loop: &ActiveEventLoop) {
        self.suspend_all(event_loop);
    }

    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: AppEvents) {
        match event {
            AppEvents::CreateWindow(attr) => {
                self.create_window(event_loop, attr);
            },
            AppEvents::KillWindow(id) => {
                self.destroy_window(event_loop, id);
            }
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, window_id: WindowId, event: WindowEvent) {
        let window_index = match self.windows.iter()
            .position(|window| window.inner?.id() == window_id)
            .ok_or(WindowAccessError::WindowNotFoundError(window_id.into())) {
            Ok(idx) => idx,
            Err(error) => {
                self.error_callback(error.into());
                return;
            },
        };
        
        match event {
            WindowEvent::CloseRequested => {
                println!("The close button was pressed; stopping");
                if self.windows.len() > 1 {
                    drop(self.windows.swap_remove(window_index));
                }
                else {
                    event_loop.exit();
                }
            },
            WindowEvent::RedrawRequested => {
                self.windows[window_index]
                    .inner
                    .
                    .as_mut()
                    .unwrap()
                    .request_redraw();
            },
            _ => (),
        };
    }
}