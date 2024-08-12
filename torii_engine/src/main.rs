use winit::application::ApplicationHandler;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::event::WindowEvent;
use winit::window::{Window, WindowId};
use winit::dpi::{LogicalSize};

use ash::vk;
use std::ffi::{CString, c_void};
use std::ptr;

mod windowhandler;

mod utility;
use utility::constants::*;
use utility::validation::*;

const VALIDATION: ValidationInfo = ValidationInfo {
    is_enable: true,
    required_validation_layers: ["VK_LAYER_KHRONOS_validation"]
};

#[derive(Default)]
struct VulkanApp {
    event_loop: Option<EventLoop<()>>,
    vk_handler: Option<VulkanHandler>,
    app_handler: Option<VulkanAppHandler>,
}

#[derive(Default)]
struct VulkanHandler {
    _entry: Option<ash::Entry>,
    instance: Option<ash::Instance>,
    debug_utils_loader: Option<ash::ext::debug_utils::Instance>,
    debug_messenger: Option<vk::DebugUtilsMessengerEXT>,
}

#[derive(Default)]
struct VulkanAppHandler {
    window: Option<Window>,
}

impl VulkanApp {
    fn init(&mut self) {
        self.event_loop = Some(EventLoop::new().expect("Couldn't create EventLoop!"));
        self.event_loop.as_ref().expect("Couldn't access EventLoop to set control flow")
            .set_control_flow(ControlFlow::Poll);
        
        self.app_handler = Some(VulkanAppHandler::default());
        self.vk_handler = Some(VulkanHandler::new());
    }
    
    fn main(&mut self) {
        // HOLY SHIT THIS TOOK SO LONG  
        self.event_loop.take().expect("Unable to take event loop")
            .run_app(self.app_handler.as_mut().expect("App handler not instantiated"))
            .unwrap();
    }
    
    pub fn run(&mut self) {
        self.init();
        self.main();
    }
}

impl Drop for VulkanApp {
    fn drop(&mut self) {
    }
}

impl VulkanHandler {
    pub fn new() -> Self {
        let mut out = VulkanHandler::default();
        
        let entry = ash::Entry::linked();
        let instance = Self::create_instance(&entry);
        let (debug_utils_loader, debug_messenger) = utility::validation::setup_debug_utils(VALIDATION, &entry, &instance);
        
        out._entry = Some(entry);
        out.instance = Some(instance);
        out.debug_utils_loader = Some(debug_utils_loader);
        out.debug_messenger = Some(debug_messenger);
        
        out
    }
    
    fn create_instance(entry: &ash::Entry) -> ash::Instance {
        if VALIDATION.is_enable && utility::validation::check_validation_layer_support(VALIDATION, entry) == false {
            panic!("Validation layers requested, but not available!");
        }
        
        let app_name = CString::new(WINDOW_TITLE).unwrap();
        let engine_name = CString::new("Gateway Engine").unwrap();
        let app_info = vk::ApplicationInfo {
            s_type: vk::StructureType::APPLICATION_INFO,
            p_next: ptr::null(),
            p_application_name: app_name.as_ptr(),
            application_version: APPLICATION_VERSION,
            p_engine_name: engine_name.as_ptr(),
            engine_version: ENGINE_VERSION,
            api_version: API_VERSION,
            _marker: Default::default(),
        };
        
        let debug_utils_create_info = utility::validation::populate_debug_messenger_create_info();
        
        let extension_names = utility::platforms::required_extension_names();
        
        let required_validation_layer_raw_names: Vec<CString> = VALIDATION
            .required_validation_layers
            .iter()
            .map(|layer_name| CString::new(*layer_name).unwrap())
            .collect();
        let enable_layer_names: Vec<*const i8> = required_validation_layer_raw_names
            .iter()
            .map(|layer_name| layer_name.as_ptr())
            .collect();
        
        let create_info = vk::InstanceCreateInfo {
            s_type: vk::StructureType::INSTANCE_CREATE_INFO,
            p_next: if VALIDATION.is_enable {
                &debug_utils_create_info as *const vk::DebugUtilsMessengerCreateInfoEXT as *const c_void
            } else {
                ptr::null()
            },
            flags: vk::InstanceCreateFlags::empty(),
            p_application_info: &app_info,
            pp_enabled_layer_names: if VALIDATION.is_enable {
                enable_layer_names.as_ptr()
            } else {
                ptr::null()
            },
            enabled_layer_count: if VALIDATION.is_enable {
                enable_layer_names.len()
            }else {
                0
            } as u32,
            pp_enabled_extension_names: extension_names.as_ptr(),
            enabled_extension_count: extension_names.len() as u32,
            _marker: Default::default(),
        };
        
        let instance: ash::Instance = unsafe {
            entry.create_instance(&create_info, None).expect("Failed to create instance!")
        };
        
        instance
    }
}

impl Drop for VulkanHandler {
    fn drop(&mut self) {
        unsafe {
            if VALIDATION.is_enable {
                self.debug_utils_loader.take().unwrap()
                    .destroy_debug_utils_messenger(self.debug_messenger.take().unwrap(), None);
            }
            self.instance.take().unwrap().destroy_instance(None);
        }
    }
}

impl ApplicationHandler for VulkanAppHandler {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window_attributes = Window::default_attributes()
            .with_title(WINDOW_TITLE)
            .with_inner_size(LogicalSize::new(WINDOW_WIDTH, WINDOW_HEIGHT));
        self.window = Some(event_loop.create_window(window_attributes).unwrap())
    }
    fn window_event(&mut self, event_loop: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                println!("The close button was pressed; stopping");
                event_loop.exit();
            },
            WindowEvent::RedrawRequested => {
                self.window.as_ref().unwrap().request_redraw();
            },
            _ => (),
        }
    }
}

fn main() {
    let mut app: VulkanApp = VulkanApp::default();
    app.run();
}
