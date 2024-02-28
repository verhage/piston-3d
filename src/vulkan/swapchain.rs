use anyhow::Result;
use ash::extensions::khr::Swapchain;
use ash::vk::{
    ColorSpaceKHR, CompositeAlphaFlagsKHR, Extent2D, Format, Image, ImageUsageFlags,
    PhysicalDevice, PresentModeKHR, SharingMode, SurfaceCapabilitiesKHR, SurfaceFormatKHR,
    SwapchainCreateFlagsKHR, SwapchainCreateInfoKHR, SwapchainKHR,
};
use ash::{Device, Instance};
use num_traits::clamp;

use crate::constants::{WINDOW_HEIGHT, WINDOW_WIDTH};
use crate::vulkan::device::QueueFamilyIndices;
use crate::vulkan::surface::SurfaceEntities;

pub struct SwapchainSupportDetails {
    capabilities: SurfaceCapabilitiesKHR,
    pub(crate) formats: Vec<SurfaceFormatKHR>,
    pub(crate) present_modes: Vec<PresentModeKHR>,
}

pub struct SwapchainEntities {
    pub swapchain_loader: Swapchain,
    pub swapchain: SwapchainKHR,
    pub swapchain_images: Vec<Image>,
    pub swapchain_format: Format,
    pub swapchain_extent: Extent2D,
}

pub fn get_swapchain_support_details(
    physical_device: PhysicalDevice,
    surface_entities: &SurfaceEntities,
) -> anyhow::Result<SwapchainSupportDetails> {
    let capabilities = unsafe {
        surface_entities
            .surface_loader
            .get_physical_device_surface_capabilities(physical_device, surface_entities.surface)
    }?;

    let formats = unsafe {
        surface_entities
            .surface_loader
            .get_physical_device_surface_formats(physical_device, surface_entities.surface)
    }?;

    let present_modes = unsafe {
        surface_entities
            .surface_loader
            .get_physical_device_surface_present_modes(physical_device, surface_entities.surface)
    }?;

    Ok(SwapchainSupportDetails {
        capabilities,
        formats,
        present_modes,
    })
}

pub fn create_swapchain(
    instance: &Instance,
    device: &Device,
    physical_device: PhysicalDevice,
    surface_entities: &SurfaceEntities,
    queue_family_indices: &QueueFamilyIndices,
) -> Result<SwapchainEntities> {
    let swapchain_support_details =
        get_swapchain_support_details(physical_device, surface_entities)?;
    let surface_format = select_surface_format(&swapchain_support_details.formats);
    let present_mode = select_present_mode(&swapchain_support_details.present_modes);
    let extent = select_swapchain_extent(&swapchain_support_details.capabilities);

    let image_count = swapchain_support_details.capabilities.min_image_count + 1;
    let image_count = if swapchain_support_details.capabilities.max_image_count > 1 {
        image_count.min(swapchain_support_details.capabilities.max_image_count)
    } else {
        image_count
    };

    let (image_sharing_mode, queue_family_indices) = if queue_family_indices.graphics_family_index
        != queue_family_indices.present_family_index
    {
        (
            SharingMode::CONCURRENT,
            vec![
                queue_family_indices.graphics_family_index.unwrap(),
                queue_family_indices.present_family_index.unwrap(),
            ],
        )
    } else {
        (SharingMode::EXCLUSIVE, vec![])
    };

    let swapchain_create_info = SwapchainCreateInfoKHR::builder()
        .flags(SwapchainCreateFlagsKHR::empty())
        .surface(surface_entities.surface)
        .min_image_count(image_count)
        .image_color_space(surface_format.color_space)
        .image_format(surface_format.format)
        .image_extent(extent)
        .image_usage(ImageUsageFlags::COLOR_ATTACHMENT)
        .image_sharing_mode(image_sharing_mode)
        .queue_family_indices(&queue_family_indices)
        .pre_transform(swapchain_support_details.capabilities.current_transform)
        .composite_alpha(CompositeAlphaFlagsKHR::OPAQUE)
        .present_mode(present_mode)
        .clipped(true)
        .old_swapchain(SwapchainKHR::null())
        .image_array_layers(1)
        .build();

    let swapchain_loader = Swapchain::new(instance, device);
    let swapchain = unsafe { swapchain_loader.create_swapchain(&swapchain_create_info, None) }?;
    let swapchain_images = unsafe { swapchain_loader.get_swapchain_images(swapchain) }?;

    Ok(SwapchainEntities {
        swapchain_loader,
        swapchain,
        swapchain_images,
        swapchain_format: surface_format.format,
        swapchain_extent: extent,
    })
}

fn select_surface_format(available_formats: &Vec<SurfaceFormatKHR>) -> SurfaceFormatKHR {
    for available_format in available_formats {
        if available_format.format == Format::B8G8R8_SRGB
            && available_format.color_space == ColorSpaceKHR::SRGB_NONLINEAR
        {
            return available_format.clone();
        }
    }

    available_formats.first().unwrap().clone()
}

fn select_present_mode(present_modes: &Vec<PresentModeKHR>) -> PresentModeKHR {
    for mode in present_modes {
        if *mode == PresentModeKHR::MAILBOX {
            return *mode;
        }
    }

    PresentModeKHR::FIFO
}

fn select_swapchain_extent(capabilities: &SurfaceCapabilitiesKHR) -> Extent2D {
    if capabilities.current_extent.width != u32::MAX {
        capabilities.current_extent
    } else {
        Extent2D {
            width: clamp(
                WINDOW_WIDTH,
                capabilities.current_extent.width,
                capabilities.current_extent.height,
            ),
            height: clamp(
                WINDOW_HEIGHT,
                capabilities.current_extent.height,
                capabilities.current_extent.height,
            ),
        }
    }
}
