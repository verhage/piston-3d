use std::mem::transmute;

use anyhow::Result;
use ash::{Entry, Instance};
use ash::extensions::ext::MetalSurface;
use ash::extensions::khr::Surface;
use ash::vk::{MetalSurfaceCreateInfoEXT, SurfaceKHR};
use cocoa::appkit::{NSView, NSWindow};
use cocoa::base::id;
use metal::foreign_types::ForeignTypeRef;
use metal::MetalLayer;
use winit::window::Window;

pub fn create_surface(
    entry: &Entry,
    instance: &Instance,
    window: &Window,
) -> Result<(Surface, SurfaceKHR)> {
    let surface_loader = Surface::new(entry, instance);
    let surface = unsafe { create_macos_surface(entry, instance, window) }?;

    Ok((surface_loader, surface))
}

unsafe fn create_macos_surface(
    entry: &Entry,
    instance: &Instance,
    window: &Window,
) -> Result<SurfaceKHR> {
    let layer = MetalLayer::new();
    layer.set_edge_antialiasing_mask(0);
    layer.set_presents_with_transaction(false);
    layer.remove_all_animations();

    let window_cocoa_id: id = unsafe { transmute(window.id()) };
    let view = window_cocoa_id.contentView();

    layer.set_contents_scale(view.backingScaleFactor());
    view.setLayer(transmute(layer.as_ptr()));
    view.setWantsLayer(true);

    let metal_surface_create_info = MetalSurfaceCreateInfoEXT::builder()
        .layer(transmute(layer.as_ptr()))
        .build();

    let metal_surface_loader = MetalSurface::new(entry, instance);
    Ok(metal_surface_loader
        .create_metal_surface(&metal_surface_create_info, None)
        .expect("Failed to create Metal surface"))
}
