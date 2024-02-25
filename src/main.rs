use anyhow::{Context, Result};
use ash::extensions::ext::DebugUtils;
use ash::vk::{
    BufferCreateInfo, CommandBuffer, CommandBufferAllocateInfo, CommandBufferBeginInfo,
    CommandBufferLevel, CommandPool, CommandPoolCreateInfo, DeviceQueueCreateInfo, DeviceSize,
    FenceCreateInfo, KhrGetPhysicalDeviceProperties2Fn, KhrPortabilityEnumerationFn,
    PhysicalDevice, Queue, SubmitInfo,
};
use ash::{self, vk, Device, Entry, Instance};
use gpu_allocator::vulkan::{
    AllocationCreateDesc, AllocationScheme, Allocator, AllocatorCreateDesc,
};
use gpu_allocator::MemoryLocation;
use raw_window_handle::HasRawDisplayHandle;
use std::borrow::Cow;
use std::ffi::CStr;
use vk::{
    DebugUtilsMessageSeverityFlagsEXT, DebugUtilsMessageTypeFlagsEXT,
    DebugUtilsMessengerCallbackDataEXT,
};
use winit::dpi::LogicalSize;
use winit::event_loop::EventLoop;
use winit::window::{Window, WindowBuilder};

fn main() -> Result<()> {
    let width = 4;
    let height = 4;
    let num_values = width * height;
    let value = 16;

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Piston")
        .with_inner_size(LogicalSize::new(1024, 768))
        .build(&event_loop)
        .unwrap();

    let entry = unsafe { ash::Entry::load() }?;
    let instance = create_instance(&entry, &window)?;

    let debug_info = vk::DebugUtilsMessengerCreateInfoEXT::builder()
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
        .pfn_user_callback(Some(vulkan_debug_callback));

    let debug_utils_loader = DebugUtils::new(&entry, &instance);
    let debug_call_back =
        unsafe { debug_utils_loader.create_debug_utils_messenger(&debug_info, None) }?;

    let physical_device = create_physical_device(&instance)?;
    let device = create_device(&instance, &physical_device)?;

    let queue = get_queue_from_device(&device, 0, 0);

    let mut allocator = {
        Allocator::new(&AllocatorCreateDesc {
            instance: instance.clone(),
            device: device.clone(),
            physical_device,
            debug_settings: Default::default(),
            buffer_device_address: false,
            allocation_sizes: Default::default(),
        })
    }?;

    let allocation;
    let buffer = {
        let create_info = BufferCreateInfo::builder()
            .size(num_values * std::mem::size_of::<u32>() as DeviceSize)
            .usage(vk::BufferUsageFlags::TRANSFER_DST);
        let buffer = unsafe { device.create_buffer(&create_info, None) }?;
        let buffer_memory_requirements = unsafe { device.get_buffer_memory_requirements(buffer) };
        allocation = allocator.allocate(&AllocationCreateDesc {
            name: "Buffer allocation",
            requirements: buffer_memory_requirements,
            location: MemoryLocation::GpuToCpu,
            linear: true,
            allocation_scheme: AllocationScheme::GpuAllocatorManaged,
        })?;
        unsafe { device.bind_buffer_memory(buffer, allocation.memory(), allocation.offset()) }?;
        buffer
    };

    let command_pool = create_command_pool(&device, 0)?;
    let command_buffer = allocate_command_buffer(&device, command_pool)?;

    let command_buffer_begin_info = CommandBufferBeginInfo::builder();
    unsafe { device.begin_command_buffer(command_buffer, &command_buffer_begin_info) }?;
    unsafe {
        device.cmd_fill_buffer(
            command_buffer,
            buffer,
            allocation.offset(),
            allocation.size(),
            value,
        )
    }
    unsafe { device.end_command_buffer(command_buffer) }?;

    let fence = {
        let fence_create_info = FenceCreateInfo::builder();
        unsafe { device.create_fence(&fence_create_info, None) }?
    };
    {
        let submit_info =
            SubmitInfo::builder().command_buffers(std::slice::from_ref(&command_buffer));
        unsafe { device.queue_submit(queue, std::slice::from_ref(&submit_info), fence) }?;
    }

    unsafe { device.wait_for_fences(std::slice::from_ref(&fence), true, u64::MAX) }?;

    let data: &[u32] = bytemuck::cast_slice(allocation.mapped_slice().context("Cannot read from host buffer")?);
    for value in data {
        println!("{}", value);
    }

    unsafe {
        debug_utils_loader.destroy_debug_utils_messenger(debug_call_back, None);
        device.destroy_fence(fence, None);
        device.destroy_command_pool(command_pool, None);
        allocator.free(allocation)?;
        device.destroy_buffer(buffer, None);
        device.destroy_device(None);
        instance.destroy_instance(None);
    }
    Ok(())
}

fn allocate_command_buffer(device: &Device, command_pool: CommandPool) -> Result<CommandBuffer> {
    let allocate_info = CommandBufferAllocateInfo::builder()
        .command_pool(command_pool)
        .command_buffer_count(1)
        .level(CommandBufferLevel::PRIMARY);
    unsafe { device.allocate_command_buffers(&allocate_info) }?
        .into_iter()
        .next()
        .context("No command buffer found")
}

fn create_command_pool(device: &Device, queue_family_index: u32) -> Result<CommandPool> {
    let create_info = CommandPoolCreateInfo::builder().queue_family_index(queue_family_index);
    Ok(unsafe { device.create_command_pool(&create_info, None) }?)
}

fn create_instance(entry: &Entry, window: &Window) -> Result<Instance> {
    let application_info = vk::ApplicationInfo::builder().api_version(vk::API_VERSION_1_3);
    let create_flags = vk::InstanceCreateFlags::ENUMERATE_PORTABILITY_KHR;
    let mut extension_names =
        ash_window::enumerate_required_extensions(window.raw_display_handle())
            .unwrap()
            .to_vec();
    extension_names.push(DebugUtils::name().as_ptr());
    extension_names.push(KhrPortabilityEnumerationFn::name().as_ptr());
    extension_names.push(KhrGetPhysicalDeviceProperties2Fn::name().as_ptr());

    let create_info = vk::InstanceCreateInfo::builder()
        .application_info(&application_info)
        .enabled_extension_names(&extension_names)
        .flags(create_flags);

    Ok(unsafe { entry.create_instance(&create_info, None) }.unwrap())
}

fn create_physical_device(instance: &Instance) -> Result<PhysicalDevice> {
    unsafe { instance.enumerate_physical_devices() }?
        .into_iter()
        .next()
        .context("No physical device found")
}

fn create_device(instance: &Instance, physical_device: &PhysicalDevice) -> Result<Device> {
    let queue_create_info = DeviceQueueCreateInfo::builder()
        .queue_family_index(0)
        .queue_priorities(&[1.0]);

    let create_info = vk::DeviceCreateInfo::builder()
        .queue_create_infos(std::slice::from_ref(&queue_create_info));
    Ok(unsafe { instance.create_device(*physical_device, &create_info, None) }.unwrap())
}

fn get_queue_from_device(device: &Device, queue_family_index: u32, queue_index: u32) -> Queue {
    unsafe { device.get_device_queue(queue_family_index, queue_index) }
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

    println!(
        "{message_severity:?}: {message_type:?} [{message_id_name} ({message_id_number})] : {message}",
    );

    vk::FALSE
}
