use std::ffi::CString;
use std::os::raw::c_void;
use std::ptr;

use crate::constants::{APPLICATION_NAME, APPLICATION_VERSION, ENGINE_NAME, VULKAN_API_VERSION};
use crate::util::debug::{create_debug_info, ValidationInfo};
use crate::util::util::vk_to_string;
use ash::extensions::ext::{DebugUtils, MetalSurface};
use ash::extensions::khr::Surface;
use ash::vk::{
    DebugUtilsMessengerCreateInfoEXT, InstanceCreateFlags, InstanceCreateInfo,
    KhrGetPhysicalDeviceProperties2Fn, KhrPortabilityEnumerationFn, StructureType,
};
use ash::{vk, Entry, Instance};

pub fn create_instance(
    entry: &Entry,
    validation_info: &ValidationInfo,
) -> anyhow::Result<Instance> {
    if validation_info.is_enabled && !is_validation_layer_supported(entry, validation_info) {
        panic!("Validation layers requested, but not available!")
    }

    let application_name = &CString::new(APPLICATION_NAME)?;
    let engine_name = &CString::new(ENGINE_NAME)?;
    let application_info = vk::ApplicationInfo::builder()
        .application_name(application_name)
        .application_version(APPLICATION_VERSION)
        .engine_name(engine_name)
        .api_version(VULKAN_API_VERSION)
        .build();

    let extension_names = vec![
        DebugUtils::name().as_ptr(),
        KhrPortabilityEnumerationFn::name().as_ptr(),
        KhrGetPhysicalDeviceProperties2Fn::name().as_ptr(),
        MetalSurface::name().as_ptr(),
        Surface::name().as_ptr(),
    ];

    let required_validation_layer_names :Vec<CString> = validation_info.required_validation_layers
        .to_vec()
        .iter()
        .map(|layer_name| CString::new(*layer_name).unwrap())
        .collect();

    let layer_names: Vec<*const i8> = required_validation_layer_names
        .iter()
        .map(|layer_name| layer_name.as_ptr())
        .collect();

    let debug_util_messenger_create_info = create_debug_info();

    let create_info = InstanceCreateInfo {
        s_type: StructureType::INSTANCE_CREATE_INFO,
        p_next: if validation_info.is_enabled {
            &debug_util_messenger_create_info as *const DebugUtilsMessengerCreateInfoEXT
                as *const c_void
        } else {
            ptr::null()
        },
        flags: InstanceCreateFlags::ENUMERATE_PORTABILITY_KHR,
        p_application_info: &application_info,
        enabled_layer_count: if validation_info.is_enabled {
            layer_names.len() as u32
        } else {
            0
        },
        pp_enabled_layer_names: if validation_info.is_enabled {
            layer_names.as_ptr()
        } else {
            ptr::null()
        },
        enabled_extension_count: extension_names.len() as u32,
        pp_enabled_extension_names: extension_names.as_ptr(),
    };

    Ok(unsafe { entry.create_instance(&create_info, None) }.expect("Error creating instance"))
}

fn is_validation_layer_supported(entry: &Entry, validation_info: &ValidationInfo) -> bool {
    let layer_properties = entry
        .enumerate_instance_layer_properties()
        .expect("Failed to enumerate layer properties");
    let mut layer_found = false;

    for required_layer_name in validation_info.required_validation_layers.iter() {
        for layer_property in layer_properties.iter() {
            let layer_name = vk_to_string(&layer_property.layer_name);
            if *required_layer_name == layer_name {
                layer_found = true;
                break;
            }
        }
    }

    layer_found
}
