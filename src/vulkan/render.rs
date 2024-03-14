use anyhow::Result;
use ash::vk::{
    AttachmentDescription, AttachmentDescriptionFlags, AttachmentLoadOp, AttachmentReference,
    AttachmentStoreOp, Format, ImageLayout, PipelineBindPoint, RenderPass, RenderPassCreateFlags,
    RenderPassCreateInfo, SampleCountFlags, SubpassDescription, SubpassDescriptionFlags,
};
use ash::Device;

pub fn create_render_pass(device: &Device, surface_format: Format) -> Result<RenderPass> {
    let color_attachment = AttachmentDescription::builder()
        .flags(AttachmentDescriptionFlags::empty())
        .format(surface_format)
        .samples(SampleCountFlags::TYPE_1)
        .load_op(AttachmentLoadOp::CLEAR)
        .store_op(AttachmentStoreOp::STORE)
        .stencil_load_op(AttachmentLoadOp::DONT_CARE)
        .stencil_store_op(AttachmentStoreOp::DONT_CARE)
        .initial_layout(ImageLayout::UNDEFINED)
        .final_layout(ImageLayout::PRESENT_SRC_KHR)
        .build();

    let color_attachment_ref = AttachmentReference::builder()
        .attachment(0)
        .layout(ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
        .build();

    let subpass = SubpassDescription::builder()
        .flags(SubpassDescriptionFlags::empty())
        .pipeline_bind_point(PipelineBindPoint::GRAPHICS)
        .color_attachments(&[color_attachment_ref])
        .build();

    let render_pass_create_info = RenderPassCreateInfo::builder()
        .flags(RenderPassCreateFlags::empty())
        .attachments(&[color_attachment])
        .subpasses(&[subpass])
        .build();

    Ok(unsafe { device.create_render_pass(&render_pass_create_info, None) }?)
}
