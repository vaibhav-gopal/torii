pub use winit::window::{Window, WindowAttributes, WindowId};
use winit::application::ApplicationHandler;
pub use winit::dpi::{LogicalPosition, LogicalSize};
use winit::event_loop::{ActiveEventLoop, EventLoop, EventLoopProxy};
use winit::event::{WindowEvent};

mod error;
pub use error::*;

mod windows;
pub use windows::*;

pub struct AppWindowOptions {
    pub init_first_window: bool,
    pub resume_mask: WindowIdentifierMask,
}

impl Default for AppWindowOptions {
    fn default() -> Self {
        AppWindowOptions {
            init_first_window: true,
            resume_mask: 0b1
        }
    }
}

pub enum AppEvents {
    CreateWindow,
    CreateAndResumeWindow(WindowIdentifier),
    ResumeWindow(WindowIdentifier),
    SuspendWindow(WindowIdentifier),
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
    
    fn init(&mut self) -> Result<()> {
        if (self.window_options.init_first_window) {
            self.windows.push(self.window_builder.build());
        }
        Ok(())
    }
    
    pub fn run_loop(&mut self) -> Result<()> {
        self.init()?;
        self.event_loop.take()
            .ok_or(StartLoopError::EventLoopAlreadyConsumedError)?
            .run_app(&mut self)
            .map_err(|e| StartLoopError::EventLoopRunAppError(e))?;
        Ok(())
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
    
    fn create_window(&mut self) -> Result<()> {
        let mut window_obj = self.window_builder.build();
        self.windows.push(window_obj);
        Ok(())
    }

    fn create_and_resume_window(&mut self, event_loop: &ActiveEventLoop) -> Result<()> {
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
        if let Err(err) = self.resume_window(event_loop, WindowIdentifier::Mask(self.window_options.resume_mask)) {
            self.error_callback(err);
        }
    }

    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: AppEvents) {
        let result = match event {
            AppEvents::CreateWindow => {
                self.create_window()
            },
            AppEvents::CreateAndResumeWindow(id) => {
                self.create_and_resume_window(event_loop)
            },
            AppEvents::ResumeWindow(id) => {
                self.resume_window(event_loop, id)
            },
            AppEvents::SuspendWindow(id) => { 
                self.suspend_window(id)
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