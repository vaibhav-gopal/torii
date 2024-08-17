use winit::window::{Window, WindowId};
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event_loop::{ActiveEventLoop, EventLoop, EventLoopProxy};
use winit::event::WindowEvent;

mod error;
pub use error::*;

#[derive(Copy, Clone)]
pub struct WindowDetails {
    pub window_title: &'static str,
    pub window_height: u32,
    pub window_width: u32,
}
impl Default for WindowDetails{
    fn default() -> Self {
        WindowDetails {
            window_title: "Torii Application",
            window_height: 800,
            window_width: 600,
        }
    }
}

pub enum AppEvents {
    CreateWindow,
}

pub struct AppHandler {
    event_loop: Option<EventLoop<AppEvents>>,
    event_loop_proxy: EventLoopProxy<AppEvents>,
    window_details: Option<WindowDetails>,
    windows: Vec<Window>,
    error_callback: Option<Box<dyn FnMut(Error)>>,
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
            window_details: None,
            windows: vec![],
            error_callback: None,
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
    pub fn set_window_details(&mut self, window_details: Option<WindowDetails>) {
        self.window_details = window_details;
    }
    pub fn window_details(&self) -> &Option<WindowDetails> {
        &self.window_details
    }
    pub fn windows(&self) -> &Vec<Window> {
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
    fn create_window(&mut self, event_loop: &ActiveEventLoop) {
        let window_details = self.window_details().unwrap_or_default();

        let window_attributes = Window::default_attributes()
            .with_title(window_details.window_title)
            .with_inner_size(LogicalSize::new(window_details.window_width, window_details.window_height));

        let window_result = event_loop
            .create_window(window_attributes)
            .map_err(|e| WindowCreationError::OSWindowCreationError(e));

        match window_result {
            Ok(window) => {
                self.windows.push(window);
            },
            Err(error) => {
                self.error_callback(error.into());
            }
        };
    }
}

impl ApplicationHandler<AppEvents> for AppHandler {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.create_window(event_loop);
    }

    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: AppEvents) {
        match event {
            AppEvents::CreateWindow => {
                self.create_window(event_loop);
            },
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, window_id: WindowId, event: WindowEvent) {
        let window_index = match self.windows.iter().position(|window| window.id() == window_id)
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
                self.windows[window_index].request_redraw();
            },
            _ => (),
        };
    }
}