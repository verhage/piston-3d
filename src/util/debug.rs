use ash::extensions::ext::DebugUtils;
use ash::vk::{
    DebugUtilsMessageSeverityFlagsEXT, DebugUtilsMessageTypeFlagsEXT,
    DebugUtilsMessengerCallbackDataEXT, DebugUtilsMessengerCreateInfoEXT, DebugUtilsMessengerEXT,
    FALSE,
};
use ash::{vk, Entry, Instance};
use log::{debug, error, info, warn};
use std::borrow::Cow;
use std::ffi::CStr;

pub struct ValidationInfo {
    pub is_enabled: bool,
    pub required_validation_layers: [&'static str; 1],
}

pub fn create_debug_utils(
    entry: &Entry,
    instance: &Instance,
    validation_info: &ValidationInfo,
) -> anyhow::Result<(DebugUtils, DebugUtilsMessengerEXT)> {
    let debug_utils_loader = DebugUtils::new(&entry, &instance);
    let debug_messenger = if validation_info.is_enabled {
        unsafe { debug_utils_loader.create_debug_utils_messenger(&create_debug_info(), None) }?
    } else {
        DebugUtilsMessengerEXT::null()
    };

    Ok((debug_utils_loader, debug_messenger))
}

pub fn create_debug_info() -> DebugUtilsMessengerCreateInfoEXT {
    DebugUtilsMessengerCreateInfoEXT::builder()
        .message_severity(
            DebugUtilsMessageSeverityFlagsEXT::ERROR
                | DebugUtilsMessageSeverityFlagsEXT::WARNING
                | DebugUtilsMessageSeverityFlagsEXT::INFO,
        )
        .message_type(
            DebugUtilsMessageTypeFlagsEXT::GENERAL
                | DebugUtilsMessageTypeFlagsEXT::VALIDATION
                | DebugUtilsMessageTypeFlagsEXT::PERFORMANCE,
        )
        .pfn_user_callback(Some(vulkan_debug_callback))
        .build()
}

unsafe extern "system" fn vulkan_debug_callback(
    message_severity: DebugUtilsMessageSeverityFlagsEXT,
    message_type: DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const DebugUtilsMessengerCallbackDataEXT,
    _user_data: *mut std::os::raw::c_void,
) -> vk::Bool32 {
    let callback_data = *p_callback_data;
    let message_id_number = callback_data.message_id_number;

    let message_id_name = if callback_data.p_message_id_name.is_null() {
        Cow::from("")
    } else {
        CStr::from_ptr(callback_data.p_message_id_name).to_string_lossy()
    };

    let message = if callback_data.p_message.is_null() {
        Cow::from("")
    } else {
        CStr::from_ptr(callback_data.p_message).to_string_lossy()
    };

    let log_message = format!(
        "{:?} [{} ({})]: {}",
        message_type, message_id_name, message_id_number, message
    );
    match message_severity {
        DebugUtilsMessageSeverityFlagsEXT::VERBOSE => debug!("{}", log_message),
        DebugUtilsMessageSeverityFlagsEXT::INFO => info!("{}", log_message),
        DebugUtilsMessageSeverityFlagsEXT::WARNING => warn!("{}", log_message),
        DebugUtilsMessageSeverityFlagsEXT::ERROR => error!("{}", log_message),
        _ => error!("{}", log_message),
    }

    FALSE
}
