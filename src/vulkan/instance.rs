use std::ffi::CString;

use ash::extensions::ext::{DebugUtils, MetalSurface};
use ash::extensions::khr::Surface;
use ash::vk::{KhrGetPhysicalDeviceProperties2Fn, KhrPortabilityEnumerationFn};
use ash::{vk, Entry, Instance};

use crate::constants::{APPLICATION_NAME, APPLICATION_VERSION, ENGINE_NAME, VULKAN_API_VERSION};
use crate::util::debug::ValidationInfo;
use crate::util::util::vk_to_string;

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
        .api_version(VULKAN_API_VERSION);

    let create_flags = vk::InstanceCreateFlags::ENUMERATE_PORTABILITY_KHR;

    let extension_names = vec![
        DebugUtils::name().as_ptr(),
        KhrPortabilityEnumerationFn::name().as_ptr(),
        KhrGetPhysicalDeviceProperties2Fn::name().as_ptr(),
        MetalSurface::name().as_ptr(),
        Surface::name().as_ptr(),
    ];

    let mut layer_names: Vec<*const i8> = Vec::new();
    if validation_info.is_enabled {
        layer_names.push(validation_info.required_validation_layer.as_ptr() as *const i8);
    }

    let create_info = vk::InstanceCreateInfo::builder()
        .application_info(&application_info)
        .enabled_extension_names(&extension_names)
        .enabled_layer_names(&*layer_names)
        .flags(create_flags);

    Ok(unsafe { entry.create_instance(&create_info, None) }?)
}

fn is_validation_layer_supported(entry: &Entry, validation_info: &ValidationInfo) -> bool {
    let layer_properties = entry
        .enumerate_instance_layer_properties()
        .expect("Failed to enumerate layer properties");
    let mut layer_found = false;
    for layer_property in layer_properties.iter() {
        let layer_name = vk_to_string(&layer_property.layer_name);
        if *validation_info.required_validation_layer == layer_name {
            layer_found = true;
            break;
        }
    }

    layer_found
}
