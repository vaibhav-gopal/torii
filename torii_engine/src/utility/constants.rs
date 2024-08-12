use ash::vk;

pub const WINDOW_TITLE: &'static str = "VulkanApp";
pub const WINDOW_WIDTH: u32 = 800;
pub const WINDOW_HEIGHT: u32 = 600;
pub const APPLICATION_VERSION: u32 = vk::make_api_version(1, 1, 0, 0);
pub const ENGINE_VERSION: u32 = vk::make_api_version(1, 1, 0 , 0);
pub const API_VERSION: u32 = vk::make_api_version(1, 1, 0, 92);
