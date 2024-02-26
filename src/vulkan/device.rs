use std::ptr::null;
use anyhow::{anyhow, Error, Result};
use ash::vk::{PhysicalDevice, Queue, QueueFlags};
use ash::{vk, Instance};
use log::{error, info};
use vk::PhysicalDeviceType;

use crate::util::util::{vk_to_string, vk_version_to_string, yes_no};

struct QueueFamilyIndices {
    graphics_family_index: Option<u32>,
}

impl QueueFamilyIndices {
    pub fn new() -> QueueFamilyIndices {
        QueueFamilyIndices {
            graphics_family_index: None,
        }
    }

    pub fn is_complete(&self) -> bool {
        self.graphics_family_index.is_some()
    }
}

pub fn select_physical_device(instance: &Instance) -> Result<PhysicalDevice> {
    let physical_devices = unsafe { instance.enumerate_physical_devices() }?;
    info!(
        "{} devices (GPU) found with Vulkan support",
        physical_devices.len()
    );

    for &physical_device in physical_devices.iter() {
        if is_suitable_physical_device(instance, physical_device) {
            return Ok(physical_device);
        }
    }

    return Err(anyhow!("No suitable device with Vulkan support found"));
}

fn is_suitable_physical_device(instance: &Instance, physical_device: PhysicalDevice) -> bool {
    let device_properties = unsafe { instance.get_physical_device_properties(physical_device) };
    let device_features = unsafe { instance.get_physical_device_features(physical_device) };
    let device_queue_families =
        unsafe { instance.get_physical_device_queue_family_properties(physical_device) };

    let device_type = match device_properties.device_type {
        PhysicalDeviceType::CPU => "CPU",
        PhysicalDeviceType::INTEGRATED_GPU => "Integrated GPU",
        PhysicalDeviceType::DISCRETE_GPU => "Discrete GPU",
        PhysicalDeviceType::VIRTUAL_GPU => "Virtual GPU",
        PhysicalDeviceType::OTHER => "Unknown",
        _ => panic!(),
    };

    let device_name = vk_to_string(&device_properties.device_name);
    info!(
        "Device: {}, id: {}, type: {}",
        device_name, device_properties.device_id, device_type
    );
    info!(
        "API version: {}",
        vk_version_to_string(device_properties.api_version)
    );
    info!("Support queue family: {}", device_queue_families.len());
    info!("# queues\tGraphics\tCompute\tTransfer\tSparse Binding");

    for queue_family in device_queue_families.iter() {
        info!(
            "{}\t\t{}\t\t{}\t{}\t\t{}",
            queue_family.queue_count,
            yes_no(queue_family.queue_flags.contains(QueueFlags::GRAPHICS)),
            yes_no(queue_family.queue_flags.contains(QueueFlags::COMPUTE)),
            yes_no(queue_family.queue_flags.contains(QueueFlags::TRANSFER)),
            yes_no(
                queue_family
                    .queue_flags
                    .contains(QueueFlags::SPARSE_BINDING)
            )
        );
    }
    info!(
        "Geometry shader support: {}",
        yes_no(device_features.geometry_shader == 1)
    );

    find_queue_family(instance, &physical_device).is_complete()
}

fn find_queue_family(instance: &Instance, physical_device: &PhysicalDevice) -> QueueFamilyIndices {
    let mut queue_family_indices = QueueFamilyIndices::new();

    let queue_families = unsafe { instance.get_physical_device_queue_family_properties(*physical_device) };

    let mut index = 0;
    for queue_family in queue_families.iter() {
        if queue_family.queue_count > 0 && queue_family.queue_flags.contains(QueueFlags::GRAPHICS) {
            queue_family_indices.graphics_family_index = Some(index);
        }
        if queue_family_indices.is_complete() {
            break;
        }
        index += 1;
    }

    queue_family_indices
}
