use crate::application_handler::AppHandler;
use crate::vulkan_api::VkApp;

pub struct Application {
    app_handler: AppHandler,
    vk_handler: VkApp,
}

impl Drop for Application {
    fn drop(&mut self) {
        
    }
}

impl Application {
    pub fn new(app_handler: AppHandler, vk_handler: VkApp) -> Self {
        Application {
            app_handler,
            vk_handler,
        }
    }
    
    pub fn run(&mut self) {
        self.app_handler.start_window();
    }
}