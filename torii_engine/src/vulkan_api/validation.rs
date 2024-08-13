use std::ffi::CStr;
use ash::{ext, vk, Entry, Instance};

pub struct DebugModuleProp {
    pub required_validation_layers: [&'static str; 1],
}

impl Default for DebugModuleProp {
    fn default() -> Self {
        DebugModuleProp {
            required_validation_layers: ["VK_LAYER_KHRONOS_validation"]
        }
    }
}

pub struct DebugModule {
    debug_utils_loader: ext::debug_utils::Instance, // stores instance-level debugging functions
    debug_messenger: vk::DebugUtilsMessengerEXT, // messenger object, handles passing debug messages to debug callback
}

impl DebugModule {
    pub fn new(entry: &Entry, instance: &Instance) -> Self {
        let (debug_utils_loader, debug_messenger) = Self::setup_debug_utils(&entry, &instance);
        
        DebugModule {
            debug_utils_loader,
            debug_messenger,
        }
    }
    pub fn setup_debug_utils(entry: &Entry, instance: &Instance) -> (ext::debug_utils::Instance, vk::DebugUtilsMessengerEXT){
        let debug_utils_loader= ext::debug_utils::Instance::new(entry, instance);
        let messenger_ci = Self::populate_debug_messenger_create_info();
        let utils_messenger = unsafe {
            debug_utils_loader
                .create_debug_utils_messenger(&messenger_ci, None)
                .expect("Debug Utils Callback")
        };

        (debug_utils_loader, utils_messenger)
    }
    pub fn populate_debug_messenger_create_info() -> vk::DebugUtilsMessengerCreateInfoEXT<'static> {
        vk::DebugUtilsMessengerCreateInfoEXT::default()
            .flags(vk::DebugUtilsMessengerCreateFlagsEXT::empty())
            .message_severity(
                vk::DebugUtilsMessageSeverityFlagsEXT::WARNING | vk::DebugUtilsMessageSeverityFlagsEXT::ERROR
                //| vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE | vk::DebugUtilsMessageSeverityFlagsEXT::INFO
            )
            .message_type(
                vk::DebugUtilsMessageTypeFlagsEXT::GENERAL | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE
                    | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION
            )
            .pfn_user_callback(Some(vulkan_debug_utils_callback))
    }
}

impl Drop for DebugModule {
    fn drop(&mut self) {
        unsafe {
            self.debug_utils_loader.destroy_debug_utils_messenger(self.debug_messenger, None);
        }
    }
}


unsafe extern "system" fn vulkan_debug_utils_callback(
    message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    message_type: vk::DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _p_user_data: *mut std::os::raw::c_void,
) -> vk::Bool32 {
    let severity = match message_severity {
        vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE => "[Verbose]",
        vk::DebugUtilsMessageSeverityFlagsEXT::WARNING => "[Warning]",
        vk::DebugUtilsMessageSeverityFlagsEXT::ERROR => "[Error]",
        vk::DebugUtilsMessageSeverityFlagsEXT::INFO => "[INFO]",
        _ => "[Unknown]"
    };
    
    let types = match message_type {
        vk::DebugUtilsMessageTypeFlagsEXT::GENERAL => "[General]",
        vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE => "[Performance]",
        vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION => "[Validation]",
        _ => "[Unknown]",
    };
    
    let message = CStr::from_ptr((*p_callback_data).p_message);
    println!("[Debug]{}{}{:?}", severity, types, message);
    
    vk::FALSE
}

pub fn check_validation_layer_support(debug_module_prop: &DebugModuleProp, entry: &Entry) -> bool {
    unsafe {
        let layer_properties = entry
            .enumerate_instance_layer_properties()
            .expect("Failed to enumerate Instance Layers Properties");

        if layer_properties.len() <= 0 {
            eprintln!("No available layers.");
            return false;
        } else {
            println!("Instance Available Layers: ");
            for layer in layer_properties.iter() {
                let layer_name = super::utility::vk_to_string(&layer.layer_name);
                println!("\t{}", layer_name);
            }
        }

        for required_layer_name in debug_module_prop.required_validation_layers.iter() {
            let mut is_layer_found = false;

            for layer_property in layer_properties.iter() {
                let test_layer_name = super::utility::vk_to_string(&layer_property.layer_name);
                if (*required_layer_name) == test_layer_name {
                    is_layer_found = true;
                    break;
                }
            }

            if is_layer_found == false {
                return false;
            }
        }
    }
    true
}
