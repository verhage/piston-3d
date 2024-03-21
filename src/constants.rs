use crate::util::debug::ValidationInfo;
use ash::vk::{make_api_version, API_VERSION_1_2};

pub const APPLICATION_NAME: &str = "Piston demo";

pub const APPLICATION_VERSION: u32 = make_api_version(0, 0, 1, 0);

pub const VULKAN_API_VERSION: u32 = API_VERSION_1_2;

pub const REQUIRED_EXTENSIONS: [&str; 1] = ["VK_KHR_swapchain"];

pub const ENGINE_NAME: &str = "Piston";

pub const WINDOW_TITLE: &str = APPLICATION_NAME;

pub const WINDOW_WIDTH: u32 = 1024;

pub const WINDOW_HEIGHT: u32 = 768;

pub const VALIDATION: ValidationInfo = ValidationInfo {
    is_enabled: true,
    required_validation_layers: ["VK_LAYER_KHRONOS_validation"],
};
