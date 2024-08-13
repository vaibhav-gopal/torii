pub use ash::vk::{API_VERSION_1_0, API_VERSION_1_1, API_VERSION_1_2, API_VERSION_1_3};
pub use ash::vk::make_api_version;

pub const ENGINE_VERSION: u32 = make_api_version(0, 0, 1, 0);
pub const ENGINE_NAME: &'static str = "Torii Engine";

pub struct VkAppInfo {
    engine_version: u32,
    api_version: u32,
    application_version: u32,
    engine_name: &'static str,
    app_name: &'static str,
}

impl Default for VkAppInfo {
    fn default() -> Self {
        Self::new(make_api_version(0, 0, 1, 0), "Torii Application")
    }
}

impl VkAppInfo {
    pub fn new(application_version: u32, app_name: &'static str) -> Self {
        VkAppInfo {
            engine_version: ENGINE_VERSION,
            api_version: API_VERSION_1_3,
            application_version,
            engine_name: ENGINE_NAME,
            app_name,
        }
    }
    
    // GETTERS

    pub fn app_name(&self) -> &'static str {
        self.app_name
    }

    pub fn engine_name(&self) -> &'static str {
        self.engine_name
    }

    pub fn application_version(&self) -> u32 {
        self.application_version
    }

    pub fn api_version(&self) -> u32 {
        self.api_version
    }

    pub fn engine_version(&self) -> u32 {
        self.engine_version
    }
}