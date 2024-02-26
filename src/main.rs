use anyhow::Result;
use ash::extensions::ext::DebugUtils;
use ash::vk::{DebugUtilsMessengerEXT, PhysicalDevice};
use ash::{self, Entry, Instance};
use log::info;
use piston::constants::{
    APPLICATION_NAME, APPLICATION_VERSION, VALIDATION, VULKAN_API_VERSION, WINDOW_HEIGHT,
    WINDOW_TITLE, WINDOW_WIDTH,
};
use winit::dpi::LogicalSize;
use winit::event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::{Window, WindowBuilder};

use piston::util::debug::create_debug_utils;
use piston::util::util::vk_version_to_string;
use piston::vulkan::device::select_physical_device;
use piston::vulkan::instance::create_instance;

struct PistonApp {
    _entry: Entry,
    instance: Instance,
    _physical_device: PhysicalDevice,
    debug_utils_loader: DebugUtils,
    debug_messenger: DebugUtilsMessengerEXT,
}

impl PistonApp {
    fn create_with_window(window: &Window) -> Result<PistonApp> {
        let entry = unsafe { ash::Entry::load() }?;
        let instance = create_instance(&entry, window, &VALIDATION)?;
        let physical_device = select_physical_device(&instance)?;

        let (debug_utils_loader, debug_messenger) =
            create_debug_utils(&entry, &instance, &VALIDATION)?;

        Ok(PistonApp {
            _entry: entry,
            instance,
            _physical_device: physical_device,
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

    fn main_loop(self, event_loop: EventLoop<()>, window: Window) {
        event_loop.run(move |event, _window_target, control_flow| match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => {
                    info!("User closed window, terminating application");
                    *control_flow = ControlFlow::Exit;
                }
                WindowEvent::KeyboardInput { input, .. } => match input {
                    KeyboardInput {
                        virtual_keycode,
                        state,
                        ..
                    } => match (virtual_keycode, state) {
                        (Some(VirtualKeyCode::Escape), ElementState::Pressed) => {
                            info!("User pressed ESC, terminating application");
                            *control_flow = ControlFlow::Exit;
                        }
                        _ => {}
                    },
                },
                _ => {}
            },
            Event::MainEventsCleared => {
                window.request_redraw();
            }
            Event::RedrawRequested(_window_id) => {
                self.draw_frame();
            }
            _ => {}
        })
    }
}

impl Drop for PistonApp {
    fn drop(&mut self) {
        unsafe {
            if VALIDATION.is_enabled {
                self.debug_utils_loader
                    .destroy_debug_utils_messenger(self.debug_messenger, None);
            }
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
    let event_loop = EventLoop::new();
    let window = PistonApp::init_window(&event_loop);

    let piston_app = PistonApp::create_with_window(&window)?;
    piston_app.main_loop(event_loop, window);
    Ok(())
}
