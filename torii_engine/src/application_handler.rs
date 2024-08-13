use winit::window::{Window, WindowId};
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::event::WindowEvent;

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

pub struct AppHandler {
    event_loop: Option<EventLoop<()>>,
    window: Option<Window>,
    window_details: WindowDetails,
}

impl Default for AppHandler {
    fn default() -> Self {
        Self::new(None)
    }
}

impl AppHandler {
    pub fn new(window_details: Option<WindowDetails>) -> Self {
        let event_loop = match EventLoop::new() {
            Ok(res) => {
                EventLoop::set_control_flow(&res, winit::event_loop::ControlFlow::Poll);
                Some(res)
            },
            Err(error) => {
                panic!("Couldn't create EventLoop for AppHandler: {:?}", error);
            }
        };
        
        let window_details = window_details.unwrap_or_default();
        
        AppHandler {
            event_loop,
            window_details,
            window: None,
        }
    }
    
    pub fn start_window(&mut self) {
        let event_loop_res = self.event_loop.take()
            .expect("Unable to consume EventLoop in AppHandler, already consumed?")
            .run_app(self);
        
        match event_loop_res {
            Ok(_) => (),
            Err(error) => {
                panic!("Error while running EventLoop: {:?}", error);
            }
        }
    }
}

impl ApplicationHandler for AppHandler {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window_attributes = Window::default_attributes()
            .with_title(self.window_details.window_title)
            .with_inner_size(LogicalSize::new(self.window_details.window_width, self.window_details.window_height));
        self.window = match event_loop.create_window(window_attributes) {
            Ok(res) => Some(res),
            Err(error) => {
                panic!("Unable to create Window with EventLoop: {:?}", error);
            }, 
        }
    }
    
    fn window_event(&mut self, event_loop: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                println!("The close button was pressed; stopping");
                event_loop.exit();
            },
            WindowEvent::RedrawRequested => {
                self.window.as_ref()
                    .expect("Unable to get reference to Window object in AppHandler")
                    .request_redraw();
            },
            _ => (),
        };
    }
}