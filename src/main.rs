use anyhow::{Result};
use ash::{self, Device, Entry, Instance};
use ash::extensions::ext::DebugUtils;
use ash::extensions::khr::Surface;
use ash::vk::{DebugUtilsMessengerEXT, PhysicalDevice, Queue, SurfaceKHR};
use log::info;
use winit::dpi::LogicalSize;
use winit::event::{ElementState, Event, KeyEvent, WindowEvent};
use winit::event_loop::EventLoop;
use winit::keyboard::{Key, NamedKey};
use winit::window::{Window, WindowBuilder};

use piston::constants::{
    APPLICATION_NAME, APPLICATION_VERSION, VALIDATION, VULKAN_API_VERSION, WINDOW_HEIGHT,
    WINDOW_TITLE, WINDOW_WIDTH,
};
use piston::util::debug::create_debug_utils;
use piston::util::util::vk_version_to_string;
use piston::vulkan::device::{create_logical_device, select_physical_device};
use piston::vulkan::instance::create_instance;
use piston::vulkan::surface::create_surface;

struct PistonApp {
    _entry: Entry,
    instance: Instance,
    _physical_device: PhysicalDevice,
    device: Device,
    _graphics_queue: Queue,
    surface_loader: Surface,
    surface: SurfaceKHR,
    debug_utils_loader: DebugUtils,
    debug_messenger: DebugUtilsMessengerEXT,
}

impl PistonApp {
    fn create_with_window(window: &Window) -> Result<PistonApp> {
        let entry = unsafe { ash::Entry::load() }?;
        let instance = create_instance(&entry, &VALIDATION)?;
        let physical_device = select_physical_device(&instance)?;
        let (device, graphics_queue) = create_logical_device(&instance, &physical_device)?;
        let (surface_loader, surface) = create_surface(&entry, &instance, &window)?;
        let (debug_utils_loader, debug_messenger) =
            create_debug_utils(&entry, &instance, &VALIDATION)?;

        Ok(PistonApp {
            _entry: entry,
            instance,
            _physical_device: physical_device,
            device,
            _graphics_queue: graphics_queue,
            surface_loader,
            surface,
            debug_utils_loader,
            debug_messenger,
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
                }
                    WindowEvent::RedrawRequested => {
                    window.pre_present_notify();
                    self.draw_frame();
                }
                _ => {}
            }
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
            self.device.destroy_device(None);
            self.surface_loader.destroy_surface(self.surface, None);
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
