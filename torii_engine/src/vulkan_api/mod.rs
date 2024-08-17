pub mod validation;
pub mod utility;
pub mod versioning;

use std::ffi::{c_void, CString};
use std::ptr;

use ash::Entry;
use ash::Instance;
use ash::vk;
use ash::Device;

use ash_window;
use winit::window::Window;
use winit::raw_window_handle::{HasDisplayHandle, HasWindowHandle};

pub struct VkProp {
    pub vk_app_info: versioning::VkAppInfo,
    pub debug_module_info: Option<validation::DebugModuleProp>,
}

impl Default for VkProp {
    fn default() -> Self {
        VkProp {
            vk_app_info: versioning::VkAppInfo::default(),
            debug_module_info: Some(validation::DebugModuleProp::default()),
        }
    }
}

pub struct VkApp {
    vk_prop: VkProp,
    entry: Entry,
    instance: Option<Instance>,
    physical_device: Option<vk::PhysicalDevice>,
    
    graphics_queue: Option<vk::Queue>,
    present_queue: Option<vk::Queue>,
    device: Option<Device>,
    
    surface_loader: Option<ash::khr::surface::Instance>,
    surface: Option<vk::SurfaceKHR>,
    
    debug_module: Option<validation::DebugModule>,
}

impl VkApp {
    pub fn new(vk_api_prop: Option<VkProp>, window: &Window) -> Self {
        let vk_prop = vk_api_prop.unwrap_or_default();

        let entry = Entry::linked();
        
        let mut api = VkApp {
            vk_prop,
            entry,
            instance: None,
            physical_device: None,
            device: None,
            graphics_queue: None,
            present_queue: None,
            surface: None,
            surface_loader: None,
            debug_module: None,
        };
        
        api.attach_instance(window);
        api.create_surface(window);
        api.pick_physical_device();
        api.create_logical_device();
        
        api
    }
    
    pub fn attach_instance(&mut self, window: &Window) {
        if self.vk_prop.debug_module_info.is_some() && validation::check_validation_layer_support(self.vk_prop.debug_module_info.as_ref().unwrap(), &self.entry) == false {
            panic!("Validation layers requested, but not available!");
        }

        let app_name = CString::new(self.vk_prop.vk_app_info.app_name()).unwrap();
        let engine_name = CString::new(self.vk_prop.vk_app_info.engine_name()).unwrap();
        let mut app_info = vk::ApplicationInfo::default()
            .application_version(self.vk_prop.vk_app_info.application_version())
            .engine_version(self.vk_prop.vk_app_info.engine_version())
            .api_version(self.vk_prop.vk_app_info.api_version());
        app_info.p_application_name = app_name.as_ptr();
        app_info.p_engine_name = engine_name.as_ptr();

        let debug_utils_create_info = validation::DebugModule::populate_debug_messenger_create_info();
        let required_validation_layer_raw_names: Vec<CString> = 
            self.vk_prop.debug_module_info.as_ref().unwrap_or(&validation::DebugModuleProp::default())
            .required_validation_layers
            .iter()
            .map(|layer_name| CString::new(*layer_name).unwrap())
            .collect();
        let enable_layer_names: Vec<*const i8> = required_validation_layer_raw_names
            .iter()
            .map(|layer_name| layer_name.as_ptr())
            .collect();
        
        let extension_names = ash_window::enumerate_required_extensions(window.display_handle().unwrap().as_raw()).unwrap();
        
        let mut create_info = vk::InstanceCreateInfo::default().flags(vk::InstanceCreateFlags::empty());
        create_info.p_next = if self.vk_prop.debug_module_info.is_some() { &debug_utils_create_info as *const vk::DebugUtilsMessengerCreateInfoEXT as *const c_void} else { ptr::null() };
        create_info.p_application_info = &app_info;
        create_info.pp_enabled_layer_names = if self.vk_prop.debug_module_info.is_some() { enable_layer_names.as_ptr() } else { ptr::null() };
        create_info.enabled_layer_count = if self.vk_prop.debug_module_info.is_some() { enable_layer_names.len() } else { 0 } as u32;
        create_info.pp_enabled_extension_names = extension_names.as_ptr();
        create_info.enabled_extension_count = extension_names.len() as u32;

        let instance = unsafe {
            self.entry.create_instance(&create_info, None).expect("Failed to create instance!")
        };
        
        self.instance = Some(instance);
        self.debug_module = Some(validation::DebugModule::new(&self.entry, self.instance.as_ref().unwrap()));
    }
    
    pub fn create_surface(&mut self, window: &Window) {
        let surface = unsafe {
            ash_window::create_surface(
                &self.entry, 
                self.instance.as_ref().unwrap(),
                window.display_handle().unwrap().as_raw(),
                window.window_handle().unwrap().as_raw(),
                None
            )
        }.unwrap();
        let surface_loader = ash::khr::surface::Instance::new(&self.entry, self.instance.as_ref().unwrap());
        
        self.surface = Some(surface);
        self.surface_loader = Some(surface_loader);
    }
    
    pub fn pick_physical_device(&mut self) {
        unsafe {
            let physical_devices = self.instance.as_ref().unwrap().enumerate_physical_devices().unwrap();
            
            println!("{} devices (GPU) found with Vulkan Support", physical_devices.len());
            
            if physical_devices.len() == 0 {
                panic!("Failed to find GPUs (discrete or integrated) with Vulkan support!");
            }

            let mut chosen_device: Option<vk::PhysicalDevice>= None;
            let mut chosen_device_id = 0;
            
            for device in physical_devices {
                let (
                    device_properties, 
                    device_features, 
                    device_queue_families
                ) = self.get_physical_device_properties(device);
                
                if Self::check_physical_device(device_properties, device_features, device_queue_families) {
                    if chosen_device.is_none() {
                        chosen_device = Some(device);
                        chosen_device_id = device_properties.device_id;
                    }
                }
            }
            
            match chosen_device {
                None => panic!("Failed to find a suitable GPU!"),
                Some(_) => {
                    println!("Chose physical device with id: {}", chosen_device_id);
                    self.physical_device = chosen_device;
                },
            }
        }
    }
    
    fn get_physical_device_properties(&mut self, physical_device: vk::PhysicalDevice) -> (vk::PhysicalDeviceProperties, vk::PhysicalDeviceFeatures, Vec<vk::QueueFamilyProperties>) {
        unsafe {
            (
                self.instance.as_ref().unwrap().get_physical_device_properties(physical_device),
                self.instance.as_ref().unwrap().get_physical_device_features(physical_device),
                self.instance.as_ref().unwrap().get_physical_device_queue_family_properties(physical_device),
            )
        }
    }
    
    fn get_current_physical_device_properties(&mut self) -> (vk::PhysicalDeviceProperties, vk::PhysicalDeviceFeatures, Vec<vk::QueueFamilyProperties>) {
        unsafe {
            (
                self.instance.as_ref().unwrap().get_physical_device_properties(self.physical_device.unwrap()),
                self.instance.as_ref().unwrap().get_physical_device_features(self.physical_device.unwrap()),
                self.instance.as_ref().unwrap().get_physical_device_queue_family_properties(self.physical_device.unwrap()),
            )
        }
    }
    
    fn check_physical_device(device_properties: vk::PhysicalDeviceProperties, device_features: vk::PhysicalDeviceFeatures, device_queue_families: Vec<vk::QueueFamilyProperties>) -> bool {
        let device_type = match device_properties.device_type {
            vk::PhysicalDeviceType::CPU => "Cpu",
            vk::PhysicalDeviceType::INTEGRATED_GPU => "Integrated GPU",
            vk::PhysicalDeviceType::DISCRETE_GPU => "Discrete GPU",
            vk::PhysicalDeviceType::VIRTUAL_GPU => "Virtual GPU",
            vk::PhysicalDeviceType::OTHER => "Unknown",
            _ => panic!(),
        };
        let device_name = utility::vk_to_string(&device_properties.device_name);
        let device_id = device_properties.device_id;
        
        println!("\tDevice Name: {}, id: {}, type: {}", device_name, device_id, device_type);
        
        let api_major = vk::api_version_major(device_properties.api_version);
        let api_minor = vk::api_version_minor(device_properties.api_version);
        let api_patch = vk::api_version_patch(device_properties.api_version);
        
        println!("\tAPI Version: {}.{}.{}", api_major, api_minor, api_patch);
        
        let support_checker = |is: bool| -> &str {match is {true => "Supported", false => "Unsupported"}};
        println!("\tSupport Queue Family Count: {}", device_queue_families.len());
        println!("\t\tQueue Count\t | Graphics, Compute, Transfer, Sparse Binding");
        for queue_family in device_queue_families.iter() {
            let is_graphics = queue_family.queue_flags.contains(vk::QueueFlags::GRAPHICS);
            let is_compute = queue_family.queue_flags.contains(vk::QueueFlags::COMPUTE);
            let is_transfer = queue_family.queue_flags.contains(vk::QueueFlags::TRANSFER);
            let is_sparse = queue_family.queue_flags.contains(vk::QueueFlags::SPARSE_BINDING);
            println!("\t\t{}\t         | {}, {}, {}, {}", queue_family.queue_count, 
                     support_checker(is_graphics), support_checker(is_compute),
                     support_checker(is_transfer), support_checker(is_sparse),
            );
        }
        println!("\t\tADDITIONAL FEATURES ---------------------------------------------------------------");
        println!("\t\tQueue Count\t | Protected Binding, Video Encode, Video Decode, Optical Flow (Nvidia)");
        for &queue_family in device_queue_families.iter() {
            let is_protected = queue_family.queue_flags.contains(vk::QueueFlags::PROTECTED);
            let is_encode = queue_family.queue_flags.contains(vk::QueueFlags::VIDEO_ENCODE_KHR);
            let is_decode = queue_family.queue_flags.contains(vk::QueueFlags::VIDEO_DECODE_KHR);
            let is_optical = queue_family.queue_flags.contains(vk::QueueFlags::OPTICAL_FLOW_NV);
            println!("\t\t{}\t         | {}, {}, {}, {}", queue_family.queue_count,
                     support_checker(is_protected), support_checker(is_encode),
                     support_checker(is_decode), support_checker(is_optical)
            );
        }
        
        let geometry_shader_support = device_features.geometry_shader == 1;
        let tesselation_shader_support = device_features.tessellation_shader == 1;
        // etc....
        println!("\tGeometry Shader Support: {}", support_checker(geometry_shader_support));
        println!("\tTesselation Shader Support: {}", support_checker(tesselation_shader_support));

        device_properties.device_type == vk::PhysicalDeviceType::DISCRETE_GPU && geometry_shader_support
    }
    
    fn create_logical_device(&mut self) {
        let (_, _, device_queue_families) = self.get_current_physical_device_properties();
        
        let mut queue_create_infos: Vec<vk::DeviceQueueCreateInfo> = Vec::new();

        let graphics_queue_family_index = Self::find_graphics_queue_family(&device_queue_families);
        let mut graphics_queue_info = vk::DeviceQueueCreateInfo::default()
            .queue_family_index(graphics_queue_family_index.unwrap())
            .queue_priorities(&[1.0f32]);
        graphics_queue_info.queue_count = 1;
        queue_create_infos.push(graphics_queue_info);
        
        let present_queue_family_index = self.find_present_queue_family(&device_queue_families);
        let mut present_queue_info = vk::DeviceQueueCreateInfo::default()
            .queue_family_index(present_queue_family_index.unwrap())
            .queue_priorities(&[1.0f32]);
        present_queue_info.queue_count = 1;
        queue_create_infos.push(present_queue_info);
        
        
        let physical_device_features_to_use = vk::PhysicalDeviceFeatures::default();
        
        let mut device_create_info = vk::DeviceCreateInfo::default()
            .queue_create_infos(&queue_create_infos)
            .enabled_features(&physical_device_features_to_use);
        device_create_info.queue_create_info_count = queue_create_infos.len() as u32;
        device_create_info.enabled_extension_count = 0;
        
        let logical_device = unsafe {
            self.instance.as_ref().unwrap()
                .create_device(self.physical_device.unwrap(), &device_create_info, None)
                .expect("Failed to create logical device!")
        };
        
        let graphics_queue = unsafe {
            logical_device
                .get_device_queue(graphics_queue_family_index.unwrap(), 0)
        };
        
        self.device = Some(logical_device);
        self.graphics_queue = Some(graphics_queue);
    }
    
    fn find_graphics_queue_family(device_queue_families: &Vec<vk::QueueFamilyProperties>) -> Option<u32> {
        let mut graphics_family_index = None;
        let mut index = 0;
        for &queue_family in device_queue_families {
            if queue_family.queue_count > 0 && queue_family.queue_flags.contains(vk::QueueFlags::GRAPHICS) {
                graphics_family_index = Some(index);
                break;
            }
            index += 1;
        }
        graphics_family_index
    }

    fn find_present_queue_family(&self, device_queue_families: &Vec<vk::QueueFamilyProperties>) -> Option<u32> {
        let mut family_index = None;
        let mut index = 0;
        for &queue_family in device_queue_families {
            let is_present_support = unsafe {
                self.surface_loader.as_ref().unwrap().get_physical_device_surface_support(self.physical_device.unwrap(), index, self.surface.unwrap()).unwrap()
            };
            
            if queue_family.queue_count > 0 && is_present_support {
                family_index = Some(index);
                break;
            }
            index += 1;
        }
        family_index
    }
}

impl Drop for VkApp {
    fn drop(&mut self) {
        unsafe {
            if let Some(logical_device) = self.device.take() {
                logical_device.destroy_device(None);
            }
            
            if let Some(debug) = self.debug_module.take() {
                drop(debug);
            }
            
            if let Some(surface) = self.surface.take() {
                self.surface_loader.take().unwrap().destroy_surface(surface, None);
            }
            
            if let Some(instance) = self.instance.take() {
                instance.destroy_instance(None);
            }
        }
    }
}