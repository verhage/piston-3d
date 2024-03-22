use std::ffi::CString;
use std::io::Cursor;
use std::path::Path;

use anyhow::Result;
use ash::util::read_spv;
use ash::vk::{
    BlendFactor, BlendOp, ColorComponentFlags, CompareOp, CullModeFlags, Extent2D, FrontFace,
    GraphicsPipelineCreateInfo, LogicOp, Offset2D, Pipeline, PipelineCache,
    PipelineColorBlendAttachmentState, PipelineColorBlendStateCreateInfo,
    PipelineDepthStencilStateCreateInfo, PipelineInputAssemblyStateCreateInfo, PipelineLayout,
    PipelineLayoutCreateInfo, PipelineMultisampleStateCreateInfo,
    PipelineRasterizationStateCreateInfo, PipelineShaderStageCreateInfo,
    PipelineVertexInputStateCreateInfo, PipelineViewportStateCreateInfo, PolygonMode,
    PrimitiveTopology, Rect2D, RenderPass, SampleCountFlags, ShaderModule, ShaderModuleCreateInfo,
    ShaderStageFlags, StencilOp, StencilOpState, Viewport,
};
use ash::Device;

use crate::util::util::load_file_bytes;

pub fn create_graphics_pipeline(
    device: &Device,
    render_pass: RenderPass,
    swapchain_extent: Extent2D,
) -> Result<(Pipeline, PipelineLayout)> {
    let mut vertex_shader_file =
        Cursor::new(load_file_bytes(Path::new("shaders/build/vert-shader.spv")));
    let mut fragment_shader_file =
        Cursor::new(load_file_bytes(Path::new("shaders/build/frag-shader.spv")));

    let vertex_shader_code = read_spv(&mut vertex_shader_file)?;
    let fragment_shader_code = read_spv(&mut fragment_shader_file)?;

    let vertex_shader_module = create_shader_module(device, vertex_shader_code)?;
    let fragment_shader_module = create_shader_module(device, fragment_shader_code)?;

    let main_function = CString::new("main").unwrap();

    let shader_stages_create_info = [
        create_pipeline_shader_stage_create_info(
            &main_function,
            vertex_shader_module,
            ShaderStageFlags::VERTEX,
        ),
        create_pipeline_shader_stage_create_info(
            &main_function,
            fragment_shader_module,
            ShaderStageFlags::FRAGMENT,
        ),
    ];

    let viewports = [Viewport::builder()
        .x(0.0)
        .y(0.0)
        .width(swapchain_extent.width as f32)
        .height(swapchain_extent.height as f32)
        .min_depth(0.0)
        .max_depth(0.0)
        .build()];

    let scissors = [Rect2D::builder()
        .offset(Offset2D::builder().x(0).y(0).build())
        .extent(swapchain_extent)
        .build()];

    let viewport_state_create_info = PipelineViewportStateCreateInfo::builder()
        .scissors(&scissors)
        .viewports(&viewports)
        .build();

    let vertex_input_state_create_info = create_vertex_input_state_create_info();
    let input_assembly_state_create_info = create_input_assembly_state_create_info();
    let rasterization_state_create_info = create_rasterization_state_create_info();
    let multisample_state_create_info = create_multisample_state_create_info();
    let depth_stencil_state_create_info = create_depth_stencil_state_create_info();
    let color_blend_state_create_info = create_color_blend_state_create_info();
    let pipeline_layout = create_pipeline_layout(device)?;
    let graphics_pipeline_create_infos = [GraphicsPipelineCreateInfo::builder()
        .stages(&shader_stages_create_info)
        .vertex_input_state(&vertex_input_state_create_info)
        .input_assembly_state(&input_assembly_state_create_info)
        .viewport_state(&viewport_state_create_info)
        .rasterization_state(&rasterization_state_create_info)
        .multisample_state(&multisample_state_create_info)
        .depth_stencil_state(&depth_stencil_state_create_info)
        .color_blend_state(&color_blend_state_create_info)
        .layout(pipeline_layout)
        .render_pass(render_pass)
        .subpass(0)
        .build()];

    let pipelines = unsafe {
        device.create_graphics_pipelines(
            PipelineCache::null(),
            &graphics_pipeline_create_infos,
            None,
        )
    }
    .unwrap();

    unsafe {
        device.destroy_shader_module(vertex_shader_module, None);
        device.destroy_shader_module(fragment_shader_module, None);
    }

    Ok((pipelines[0], pipeline_layout))
}

fn create_shader_module(device: &Device, shader_code: Vec<u32>) -> Result<ShaderModule> {
    let shader_module_create_info = ShaderModuleCreateInfo::builder().code(&shader_code).build();

    Ok(unsafe { device.create_shader_module(&shader_module_create_info, None) }?)
}

fn create_pipeline_shader_stage_create_info(
    main_function_name: &CString,
    shader_module: ShaderModule,
    stage: ShaderStageFlags,
) -> PipelineShaderStageCreateInfo {
    PipelineShaderStageCreateInfo::builder()
        .module(shader_module)
        .name(main_function_name)
        .stage(stage)
        .build()
}

fn create_vertex_input_state_create_info() -> PipelineVertexInputStateCreateInfo {
    PipelineVertexInputStateCreateInfo::default()
}

fn create_input_assembly_state_create_info() -> PipelineInputAssemblyStateCreateInfo {
    PipelineInputAssemblyStateCreateInfo::builder()
        .primitive_restart_enable(false)
        .topology(PrimitiveTopology::TRIANGLE_LIST)
        .build()
}

fn create_rasterization_state_create_info() -> PipelineRasterizationStateCreateInfo {
    PipelineRasterizationStateCreateInfo::builder()
        .depth_clamp_enable(false)
        .cull_mode(CullModeFlags::BACK)
        .front_face(FrontFace::CLOCKWISE)
        .line_width(1.0)
        .polygon_mode(PolygonMode::FILL)
        .rasterizer_discard_enable(false)
        .depth_bias_clamp(0.0)
        .depth_bias_constant_factor(0.0)
        .depth_bias_enable(false)
        .depth_bias_slope_factor(0.0)
        .build()
}

fn create_multisample_state_create_info() -> PipelineMultisampleStateCreateInfo {
    PipelineMultisampleStateCreateInfo::builder()
        .rasterization_samples(SampleCountFlags::TYPE_1)
        .sample_shading_enable(false)
        .min_sample_shading(0.0)
        .alpha_to_one_enable(false)
        .alpha_to_coverage_enable(false)
        .build()
}

fn create_depth_stencil_state_create_info() -> PipelineDepthStencilStateCreateInfo {
    let stencil_state = StencilOpState::builder()
        .fail_op(StencilOp::KEEP)
        .pass_op(StencilOp::KEEP)
        .depth_fail_op(StencilOp::KEEP)
        .compare_op(CompareOp::ALWAYS)
        .compare_mask(0)
        .write_mask(0)
        .reference(0)
        .build();

    PipelineDepthStencilStateCreateInfo::builder()
        .depth_test_enable(false)
        .depth_write_enable(false)
        .depth_compare_op(CompareOp::LESS_OR_EQUAL)
        .depth_bounds_test_enable(false)
        .front(stencil_state)
        .back(stencil_state)
        .max_depth_bounds(1.0)
        .min_depth_bounds(0.0)
        .build()
}

fn create_color_blend_state_create_info() -> PipelineColorBlendStateCreateInfo {
    let color_blend_attachment_states = [PipelineColorBlendAttachmentState::builder()
        .blend_enable(false)
        .color_write_mask(ColorComponentFlags::RGBA)
        .src_color_blend_factor(BlendFactor::ONE)
        .dst_color_blend_factor(BlendFactor::ZERO)
        .color_blend_op(BlendOp::ADD)
        .src_alpha_blend_factor(BlendFactor::ONE)
        .dst_alpha_blend_factor(BlendFactor::ZERO)
        .alpha_blend_op(BlendOp::ADD)
        .build()];

    PipelineColorBlendStateCreateInfo::builder()
        .logic_op_enable(false)
        .logic_op(LogicOp::COPY)
        .attachments(&color_blend_attachment_states)
        .build()
}

fn create_pipeline_layout(device: &Device) -> Result<PipelineLayout> {
    let pipeline_layout_create_info = PipelineLayoutCreateInfo::default();
    Ok(unsafe { device.create_pipeline_layout(&pipeline_layout_create_info, None) }?)
}
