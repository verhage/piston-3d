use anyhow::Result;
use ash::extensions::ext::DebugUtils;
use ash::extensions::khr::Swapchain;
use ash::vk::{
    DebugUtilsMessengerEXT, Extent2D, Format, Image, ImageView, PhysicalDevice, Pipeline,
    PipelineLayout, Queue, RenderPass, SwapchainKHR,
};
use ash::{self, Device, Entry, Instance};
use log::info;
use winit::dpi::LogicalSize;
use winit::event::{ElementState, Event, KeyEvent, WindowEvent};
use winit::event_loop::EventLoop;
use winit::keyboard::{Key, NamedKey};
use winit::window::{Window, WindowBuilder};

use piston::constants::*;
use piston::util::debug::create_debug_utils;
use piston::util::util::vk_version_to_string;
use piston::vulkan::device::{create_logical_device, select_physical_device};
use piston::vulkan::instance::create_instance;
use piston::vulkan::pipeline::create_graphics_pipeline;
use piston::vulkan::render::create_render_pass;
use piston::vulkan::surface::{create_surface, SurfaceEntities};
use piston::vulkan::swapchain::create_swapchain;

struct PistonApp {
    _entry: Entry,
    instance: Instance,
    _physical_device: PhysicalDevice,
    device: Device,
    _graphics_queue: Queue,
    _present_queue: Queue,
    surface_entities: SurfaceEntities,
    debug_utils_loader: DebugUtils,
    debug_messenger: DebugUtilsMessengerEXT,
    swapchain_loader: Swapchain,
    swapchain: SwapchainKHR,
    _swapchain_format: Format,
    _swapchain_images: Vec<Image>,
    _swapchain_extent: Extent2D,
    swapchain_image_views: Vec<ImageView>,
    render_pass: RenderPass,
    pipeline_layout: PipelineLayout,
    pipeline: Pipeline,
}

impl PistonApp {
    fn create_with_window(window: &Window) -> Result<PistonApp> {
        let entry = unsafe { ash::Entry::load() }?;
        let instance = create_instance(&entry, &VALIDATION)?;
        let surface_entities = create_surface(&entry, &instance, &window)?;
        let physical_device = select_physical_device(&instance, &surface_entities)?;
        let (device, queue_family_indices) =
            create_logical_device(&instance, physical_device, &surface_entities)?;
        let (debug_utils_loader, debug_messenger) =
            create_debug_utils(&entry, &instance, &VALIDATION)?;
        let graphics_queue = unsafe {
            device.get_device_queue(queue_family_indices.graphics_family_index.unwrap(), 0)
        };
        let present_queue = unsafe {
            device.get_device_queue(queue_family_indices.present_family_index.unwrap(), 0)
        };

        let (swapchain_entities, swapchain_image_views) = create_swapchain(
            &instance,
            &device,
            physical_device,
            &surface_entities,
            &queue_family_indices,
        )?;

        let render_pass = create_render_pass(&device, swapchain_entities.swapchain_format)?;

        let (pipeline, pipeline_layout) =
            create_graphics_pipeline(&device, render_pass, swapchain_entities.swapchain_extent)?;

        Ok(PistonApp {
            _entry: entry,
            instance,
            _physical_device: physical_device,
            device,
            _graphics_queue: graphics_queue,
            _present_queue: present_queue,
            surface_entities,
            debug_utils_loader,
            debug_messenger,
            swapchain_loader: swapchain_entities.swapchain_loader,
            swapchain: swapchain_entities.swapchain,
            _swapchain_format: swapchain_entities.swapchain_format,
            _swapchain_images: swapchain_entities.swapchain_images,
            _swapchain_extent: swapchain_entities.swapchain_extent,
            swapchain_image_views,
            render_pass,
            pipeline_layout,
            pipeline,
        })
    }

    fn init_window(event_loop: &EventLoop<()>) -> Window {
        WindowBuilder::new()
            .with_title(WINDOW_TITLE)
            .with_inner_size(LogicalSize::new(WINDOW_WIDTH, WINDOW_HEIGHT))
            .build(&event_loop)
            .unwrap()
    }

    fn draw_frame(&self) {}

    fn main_loop(self, event_loop: EventLoop<()>, window: Window) -> Result<()> {
        let redraw_requested = false;
        let mut close_requested = false;

        Ok(event_loop.run(move |event, event_loop| match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => {
                    info!("User closed window, terminating event loop");
                    close_requested = true;
                }
                WindowEvent::KeyboardInput {
                    event:
                        KeyEvent {
                            logical_key: key,
                            state: ElementState::Pressed,
                            ..
                        },
                    ..
                } => match key.as_ref() {
                    Key::Named(NamedKey::Escape) => {
                        info!("User pressed ESC, terminating event loop");
                        close_requested = true;
                    }
                    _ => {}
                },
                WindowEvent::RedrawRequested => {
                    window.pre_present_notify();
                    self.draw_frame();
                }
                _ => {}
            },
            Event::AboutToWait => {
                if redraw_requested && !close_requested {
                    window.request_redraw()
                }
                if close_requested {
                    event_loop.exit()
                }
            }
            _ => {}
        })?)
    }
}

impl Drop for PistonApp {
    fn drop(&mut self) {
        unsafe {
            if VALIDATION.is_enabled {
                self.debug_utils_loader
                    .destroy_debug_utils_messenger(self.debug_messenger, None);
            }

            self.device.destroy_pipeline(self.pipeline, None);
            self.device
                .destroy_pipeline_layout(self.pipeline_layout, None);
            self.device.destroy_render_pass(self.render_pass, None);

            for &image_view in self.swapchain_image_views.iter() {
                self.device.destroy_image_view(image_view, None);
            }

            self.swapchain_loader
                .destroy_swapchain(self.swapchain, None);

            self.device.destroy_device(None);

            self.surface_entities
                .surface_loader
                .destroy_surface(self.surface_entities.surface, None);
            self.instance.destroy_instance(None);
        }
    }
}

fn main() -> Result<()> {
    env_logger::init();

    info!(
        "Starting {} v{}, built for Vulkan v{}",
        APPLICATION_NAME,
        vk_version_to_string(APPLICATION_VERSION),
        vk_version_to_string(VULKAN_API_VERSION)
    );
    let event_loop = EventLoop::new()?;
    let window = PistonApp::init_window(&event_loop);
    let piston_app = PistonApp::create_with_window(&window)?;
    piston_app.main_loop(event_loop, window)?;
    Ok(())
}
