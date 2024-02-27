use crate::constants::REQUIRED_EXTENSIONS;
use anyhow::{anyhow, Result};
use ash::vk::{
    DeviceCreateInfo, DeviceQueueCreateInfo, PhysicalDevice, PhysicalDeviceFeatures, Queue,
    QueueFlags,
};
use ash::{vk, Device, Instance};
use log::{debug, info};
use std::collections::HashSet;
use vk::PhysicalDeviceType;

use crate::util::util::{vk_to_string, vk_version_to_string, yes_no};
use crate::vulkan::surface::SurfaceWrapper;

pub struct Queues {
    pub graphics_queue: Queue,
    pub present_queue: Queue,
}

struct QueueFamilyIndices {
    graphics_family_index: Option<u32>,
    present_family_index: Option<u32>,
}

impl QueueFamilyIndices {
    pub fn new() -> QueueFamilyIndices {
        QueueFamilyIndices {
            graphics_family_index: None,
            present_family_index: None,
        }
    }

    pub fn is_complete(&self) -> bool {
        self.graphics_family_index.is_some() && self.present_family_index.is_some()
    }
}

pub fn select_physical_device(
    instance: &Instance,
    surface_wrapper: &SurfaceWrapper,
) -> Result<PhysicalDevice> {
    let physical_devices = unsafe { instance.enumerate_physical_devices() }?;
    info!(
        "{} devices (GPU) found with Vulkan support",
        physical_devices.len()
    );

    for &physical_device in physical_devices.iter() {
        if is_suitable_physical_device(instance, physical_device, surface_wrapper) {
            return Ok(physical_device);
        }
    }

    return Err(anyhow!("No suitable supported device found"));
}

pub fn create_logical_device(
    instance: &Instance,
    physical_device: PhysicalDevice,
    surface_wrapper: &SurfaceWrapper,
) -> Result<(Device, Queues)> {
    let queue_family_indices = find_queue_family(instance, physical_device, surface_wrapper);
    let queue_priorities = [1.0f32];
    let queue_create_info = DeviceQueueCreateInfo::builder()
        .queue_family_index(queue_family_indices.graphics_family_index.unwrap())
        .queue_priorities(&queue_priorities)
        .build();
    let physical_device_features = PhysicalDeviceFeatures::builder().build();
    let device_create_info = DeviceCreateInfo::builder()
        .queue_create_infos(&[queue_create_info])
        .enabled_features(&physical_device_features)
        .build();

    let device = unsafe { instance.create_device(physical_device, &device_create_info, None) }?;
    let graphics_queue =
        unsafe { device.get_device_queue(queue_family_indices.graphics_family_index.unwrap(), 0) };
    let present_queue =
        unsafe { device.get_device_queue(queue_family_indices.graphics_family_index.unwrap(), 0) };

    Ok((
        device,
        Queues {
            graphics_queue,
            present_queue,
        },
    ))
}

fn is_suitable_physical_device(
    instance: &Instance,
    physical_device: PhysicalDevice,
    surface_wrapper: &SurfaceWrapper,
) -> bool {
    let queue_families_ok = check_queue_families(instance, physical_device, surface_wrapper);
    let extension_support_ok = check_extension_support(instance, physical_device);

    info!("Queue families supported: {}", yes_no(queue_families_ok));
    info!(
        "Required extensions supported: {}",
        yes_no(extension_support_ok)
    );

    queue_families_ok && extension_support_ok
}

fn check_extension_support(instance: &Instance, physical_device: PhysicalDevice) -> bool {
    let available_extensions =
        unsafe { instance.enumerate_device_extension_properties(physical_device) }.unwrap();

    let mut available_extension_names = vec![];
    debug!("Available device extensions:");
    for extension in available_extensions.iter() {
        let extension_name = vk_to_string(&extension.extension_name);
        debug!(" - {} version {}", extension_name, extension.spec_version);
        available_extension_names.push(extension_name);
    }

    let mut required_extensions = HashSet::new();
    for required_extension in REQUIRED_EXTENSIONS.iter() {
        required_extensions.insert(required_extension.to_string());
    }

    for available_extension in available_extension_names.iter() {
        required_extensions.remove(available_extension);
    }

    required_extensions.is_empty()
}

fn check_queue_families(
    instance: &Instance,
    physical_device: PhysicalDevice,
    surface_wrapper: &SurfaceWrapper,
) -> bool {
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

    find_queue_family(instance, physical_device, surface_wrapper).is_complete()
}

fn find_queue_family(
    instance: &Instance,
    physical_device: PhysicalDevice,
    surface_wrapper: &SurfaceWrapper,
) -> QueueFamilyIndices {
    let mut queue_family_indices = QueueFamilyIndices::new();

    let queue_families =
        unsafe { instance.get_physical_device_queue_family_properties(physical_device) };

    let mut index = 0;
    for queue_family in queue_families.iter() {
        if queue_family.queue_count > 0 {
            if queue_family.queue_flags.contains(QueueFlags::GRAPHICS) {
                queue_family_indices.graphics_family_index = Some(index);
            }

            let is_present_supported = unsafe {
                surface_wrapper
                    .surface_loader
                    .get_physical_device_surface_support(
                        physical_device,
                        index,
                        surface_wrapper.surface,
                    )
            }
            .unwrap();
            if is_present_supported {
                queue_family_indices.present_family_index = Some(index);
            }

            if queue_family_indices.is_complete() {
                break;
            }
        }
        index += 1;
    }

    queue_family_indices
}
