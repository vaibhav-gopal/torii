pub use winit::window::{Window, WindowAttributes, WindowId};
use winit::application::ApplicationHandler;
pub use winit::dpi::{LogicalPosition, LogicalSize};
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

pub type WindowObjectBuilderPreCallback = Option<Box<dyn FnMut(&mut WindowObjectBuilder)>>;
pub type WindowObjectBuilderPostCallback = Option<Box<dyn FnMut(&mut WindowObjectBuilder, &mut WindowObject)>>;

pub struct WindowObjectBuilder {
    pub attr_state: WindowAttributes,
    pub mask_state: WindowIdentifierMask,
    pub uuid_state: WindowIdentifierUUID,
    pub prebuild_callback: WindowObjectBuilderPreCallback,
    pub postbuild_callback: WindowObjectBuilderPostCallback
}

pub enum WindowState {
    Active(Window),
    Suspended
}

pub struct WindowObject {
    pub state: WindowState,
    pub attr: WindowAttributes,
    pub mask: WindowIdentifierMask,
    pub uuid: WindowIdentifierUUID,
}

pub struct AppWindowOptions {
    pub resume_mask: WindowIdentifierMask,
}

pub enum AppEvents {
    CreateWindow,
    KillWindow(WindowIdentifier)
}

pub type AppHandlerErrorCallback = Option<Box<dyn Fn(Error)>>;

pub struct AppHandler {
    event_loop: Option<EventLoop<AppEvents>>,
    pub event_loop_proxy: EventLoopProxy<AppEvents>,
    pub error_callback: AppHandlerErrorCallback,
    pub windows: Vec<WindowObject>,
    pub window_builder: WindowObjectBuilder,
    pub window_options: AppWindowOptions
}

impl Default for WindowObjectBuilder {
    fn default() -> Self {
        WindowObjectBuilder {
            attr_state: WindowAttributes::default()
                .with_title("Torii Application")
                .with_inner_size(LogicalSize::new(800, 600))
                .with_position(LogicalPosition::new(0, 0)),
            uuid_state: 0,
            mask_state: 0b1,
            prebuild_callback: None,
            postbuild_callback: None
        }
    }
}

impl WindowObjectBuilder {
    pub fn new(attr: Option<WindowAttributes>, prebuild_callback: WindowObjectBuilderPreCallback, postbuild_callback: WindowObjectBuilderPostCallback) -> Self {
        let mut obj = WindowObjectBuilder::default();
        if let Some(attr) = attr { obj.attr_state = attr };
        obj.prebuild_callback = prebuild_callback;
        obj.postbuild_callback = postbuild_callback;
        obj
    }

    pub fn build(&mut self) -> WindowObject {
        self.run_prebuild();
        let mut obj = WindowObject {
            attr: self.attr_state.clone(),
            mask: self.mask_state,
            uuid: self.uuid_state,
            state: WindowState::Suspended,
        };
        self.run_postbuild(&mut obj);
        obj
    }
    
    pub fn run_prebuild(&mut self) {
        if let Some(mut callback) = self.prebuild_callback.take() {
            let mut callbackf = &mut callback;
            callbackf(self);
            self.prebuild_callback = Some(callback);
        }
    }
    
    pub fn run_postbuild(&mut self, obj: &mut WindowObject) {
        if let Some(mut callback) = self.postbuild_callback.take() {
            let mut callbackf = &mut callback;
            callbackf(self, obj);
            self.postbuild_callback = Some(callback);
        }
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
            window_builder: WindowObjectBuilder::default(),
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
    
    pub fn send_event(&self, event: AppEvents) -> Result<()> {
        self.event_loop_proxy.send_event(event)
            .map_err(|_| EventLoopProxyError::EventLoopProxySendEventError)?;
        Ok(())
    }

    pub fn error_callback(&self, error: Error) {
        if let Some(callback) = &self.error_callback {
            callback(error)
        }
    }

    pub fn get_window_index(&self, id: WindowIdentifier) -> Result<usize> {
        let window_index = self.windows.iter()
            .position(|obj| obj.check_match(id))
            .ok_or(WindowError::WindowNotFoundError(id).into());
        window_index
    }
    
    pub fn get_window(&self, id: WindowIdentifier) -> Result<&WindowObject> {
        let idx = self.get_window_index(id)?;
        Ok(&self.windows[idx])
    }

    pub fn get_window_mut(&mut self, id: WindowIdentifier) -> Result<&mut WindowObject> {
        let idx = self.get_window_index(id)?;
        Ok(&mut self.windows[idx])
    }
}

impl AppHandler {
    // PRIVATE FUNCTIONS (called from the event loop ; no return value)
    fn resume_window(&mut self, event_loop: &ActiveEventLoop, id: WindowIdentifier) -> Result<()> {
        let window = self.get_window_mut(id)?;
        window.resume(event_loop)?;
        Ok(())
    }
    
    fn suspend_window(&mut self, id: WindowIdentifier) -> Result<()> {
        let window = self.get_window_mut(id)?;
        window.suspend()?;
        Ok(())
    }
    
    fn create_window(&mut self, event_loop: &ActiveEventLoop) -> Result<()> {
        let mut window_obj = self.window_builder.build();
        let window = event_loop
            .create_window(window_obj.attr.clone())
            .map_err(|e| WindowError::WindowCreationError(e))?;
        window_obj.state = WindowState::Active(window);
        self.windows.push(window_obj);
        Ok(())
    }
    
    fn destroy_window(&mut self, id: WindowIdentifier) -> Result<()> {
        let idx = self.get_window_index(id)?;
        drop(self.windows.swap_remove(idx));
        Ok(())
    }
}

impl ApplicationHandler<AppEvents> for AppHandler {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.create_window(event_loop);
        // match self.resume_window(event_loop, WindowIdentifier::Mask(self.window_options.resume_mask)) {
        //     Ok(_) => {}
        //     Err(err) => {
        //         self.error_callback(err);
        //     }
        // }
    }

    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: AppEvents) {
        let result = match event {
            AppEvents::CreateWindow => {
                self.create_window(event_loop)
            },
            AppEvents::KillWindow(id) => {
                self.destroy_window(id)
            }
        };
        
        match result {
            Ok(_) => {}
            Err(err) => {
                self.error_callback(err);
            }
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, window_id: WindowId, event: WindowEvent) {
        let window_index = match self.get_window_index(WindowIdentifier::WindowId(window_id)) {
            Ok(idx) => {
                idx
            }
            Err(err) => {
                self.error_callback(err);
                return;
            }
        };
        
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

    fn suspended(&mut self, _event_loop: &ActiveEventLoop) {
        match self.suspend_window(WindowIdentifier::Mask(WindowIdentifierMask::MAX)) {
            Ok(_) => {}
            Err(err) => {
                self.error_callback(err);
            }
        }
    }
}