use winit::window::{Window, WindowAttributes, WindowId};
use winit::application::ApplicationHandler;
use winit::dpi::{LogicalPosition, LogicalSize};
use winit::event_loop::{ActiveEventLoop, EventLoop, EventLoopProxy};
use winit::event::{WindowEvent};

mod error;
pub use error::*;

pub type WindowIdentifierMask = u32;
pub type WindowIdentifierUUID = u32;

#[derive(Debug, Clone, Copy)]
pub enum WindowIdentifier {
    // User given window mask (up to 32)
    Mask(WindowIdentifierMask),
    // User given window ID
    UUID(WindowIdentifierUUID),
    // WindowId (part of winit crate)
    WindowId(WindowId)
}

pub type WindowAttributeBuilderCallback = Option<Box<dyn Fn(&WindowAttributes) -> WindowAttributes>>;

pub struct WindowAttributeBuilder {
    inner: WindowAttributes,
    callback: WindowAttributeBuilderCallback
}

enum WindowState {
    Active(Window),
    Suspended
}

struct WindowObject {
    state: WindowState,
    attr: WindowAttributes,
    mask: WindowIdentifierMask,
    uuid: WindowIdentifierUUID,
}

struct AppWindowOptions {
    resume_mask: WindowIdentifierMask
}

pub enum AppEvents {
    CreateWindow{attr: Option<WindowAttributes>, mask: WindowIdentifierMask, uuid: WindowIdentifierUUID},
    KillWindow(WindowIdentifier)
}

pub type AppHandlerErrorCallback = Option<Box<dyn Fn(Error)>>;

struct AppHandler {
    event_loop: Option<EventLoop<AppEvents>>,
    event_loop_proxy: EventLoopProxy<AppEvents>,
    error_callback: AppHandlerErrorCallback,
    windows: Vec<WindowObject>,
    attr_builder: WindowAttributeBuilder,
    window_options: AppWindowOptions
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
    pub fn new(inner: WindowAttributes, callback: WindowAttributeBuilderCallback) -> Self {
        WindowAttributeBuilder {
            inner,
            callback,
        }
    }
    
    pub fn build(&self) -> WindowAttributes {
        let mut attr = self.inner.clone();
        if let Some(callback) = &self.callback {
            attr = callback(&attr)
        }
        attr
    }

    pub fn set_inner(&mut self, inner: WindowAttributes) {
        self.inner = inner;
    }

    pub fn set_callback(&mut self, callback: WindowAttributeBuilderCallback) {
        self.callback = callback;
    }
}

impl WindowObject {
    fn resume(&mut self, event_loop: &ActiveEventLoop) -> Result<()> {
        match self.state {
            WindowState::Active(_) => {
                Ok(())
            }
            WindowState::Suspended => {
                let window = event_loop
                    .create_window(self.attr.clone())
                    .map_err(|e| WindowObjectError::WindowResumeError(WindowError::WindowCreationError(e)))?;
                self.state = WindowState::Active(window);
                Ok(())
            }
        }
    }
    fn suspend(&mut self) -> Result<()> {
        match self.state {
            WindowState::Active(_) => {
                self.state = WindowState::Suspended;
                Ok(())
            }
            WindowState::Suspended => {
                Ok(())
            }
        }
    }
    fn check_match(&self, id: WindowIdentifier) -> bool {
        match id {
            WindowIdentifier::Mask(mask) => {
                (self.mask & mask) > 1
            }
            WindowIdentifier::UUID(uuid) => {
                self.uuid == uuid
            }
            WindowIdentifier::WindowId(id) => {
                match &self.state {
                    WindowState::Active(window) => {
                        window.id() == id
                    }
                    WindowState::Suspended => {
                        false
                    }
                }
            }
        }
    }
}

impl Default for AppWindowOptions {
    fn default() -> Self {
        AppWindowOptions {
            resume_mask: 0b1
        }
    }
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
            attr_builder: WindowAttributeBuilder::default(),
            window_options: AppWindowOptions::default()
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
    pub fn set_error_callback(&mut self, error_callback: AppHandlerErrorCallback) {
        self.error_callback = error_callback;
    }
    pub fn send_event(&mut self, event: AppEvents) -> Result<()> {
        self.event_loop_proxy.send_event(event)
            .map_err(|_| EventLoopProxyError::EventLoopProxySendEventError)?;
        Ok(())
    }
}

impl AppHandler {
    // PRIVATE FUNCTIONS (called from the event loop ; no return value)
    fn error_callback(&self, error: Error) {
        match &self.error_callback {
            Some(callback) => {callback(error)}
            None => {}
        }
    }
    
    fn window_index(&self, id: WindowIdentifier) -> Result<usize> {
        let window_index = self.windows.iter()
            .position(|obj| obj.check_match(id))
            .ok_or(WindowError::WindowNotFoundError(id).into());
        window_index
    }
    
    fn resume_window(&mut self, event_loop: &ActiveEventLoop, id: WindowIdentifier) -> Result<()> {
        let idx = self.window_index(id)?;
        self.windows[idx].resume(event_loop)?;
        Ok(())
    }
    
    fn suspend_window(&mut self, id: WindowIdentifier) -> Result<()> {
        let idx = self.window_index(id)?;
        self.windows[idx].suspend()?;
        Ok(())
    }
    
    fn create_window(&mut self, event_loop: &ActiveEventLoop, attributes: Option<WindowAttributes>, mask: WindowIdentifierMask, uuid: WindowIdentifierUUID) -> Result<()> {
        let attributes = attributes.unwrap_or(self.attr_builder.build());
        let window = event_loop
            .create_window(attributes.clone())
            .map_err(|e| WindowError::WindowCreationError(e))?;
        self.windows.push(
            WindowObject {
                state: WindowState::Active(window),
                attr: attributes,
                mask,
                uuid,
            }
        );
        Ok(())
    }
    
    fn destroy_window(&mut self, id: WindowIdentifier) -> Result<()> {
        let idx = self.window_index(id)?;
        drop(self.windows.swap_remove(idx));
        Ok(())
    }
}

impl ApplicationHandler<AppEvents> for AppHandler {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        match self.resume_window(event_loop, WindowIdentifier::Mask(self.window_options.resume_mask)) {
            Ok(_) => {}
            Err(err) => {
                self.error_callback(err);
            }
        }
    }

    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: AppEvents) {
        let result;
        match event {
            AppEvents::CreateWindow{attr, mask, uuid} => {
                result = self.create_window(event_loop, attr, mask, uuid);
            },
            AppEvents::KillWindow(id) => {
                result = self.destroy_window(id);
            }
        }
        
        match result {
            Ok(_) => {}
            Err(err) => {
                self.error_callback(err);
            }
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, window_id: WindowId, event: WindowEvent) {
        let window_index;
        match self.window_index(WindowIdentifier::WindowId(window_id)) {
            Ok(idx) => {
                window_index = idx;
            }
            Err(err) => {
                self.error_callback(err);
                return;
            }
        }
        
        let window: &Window;
        if let WindowState::Active(win) = &self.windows[window_index].state {
            window = win;
        } else {
            unreachable!();
        }
        
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
                window.request_redraw();
            },
            _ => {},
        };
    }

    fn suspended(&mut self, event_loop: &ActiveEventLoop) {
        match self.suspend_window(WindowIdentifier::Mask(WindowIdentifierMask::MAX)) {
            Ok(_) => {}
            Err(err) => {
                self.error_callback(err);
            }
        }
    }
}