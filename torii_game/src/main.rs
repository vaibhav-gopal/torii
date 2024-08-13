use torii_engine::*;

fn main() {
    let window_details = application_handler::WindowDetails {
        window_title: "Insane Game",
        window_width: 1200,
        window_height: 800,
    };
    
    let vkprop = vulkan_api::VkProp {
      vk_app_info: vulkan_api::versioning::VkAppInfo::new(
          vulkan_api::versioning::make_api_version(0, 0, 1, 0), "Insane Game"
      ),
      debug_module_info: Some(vulkan_api::validation::DebugModuleProp::default()),
    };
    
    let application_handler = application_handler::AppHandler::new(Some(window_details));
    let vk_handler = vulkan_api::VkApp::new(Some(vkprop));
    
    let mut application = application::Application::new(application_handler, vk_handler);
    application.run();
}