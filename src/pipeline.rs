use align_data::{include_aligned, Align32};
use ash::{
    version::DeviceV1_0,
    vk::{self},
    Device,
};
use byte_slice_cast::AsSliceOf;
use std::ffi::CString;

use crate::vulkan_context::VulkanContext;

pub fn create_graphics_pipeline(
    context: &VulkanContext,
    render_pass: vk::RenderPass,
) -> vk::Pipeline {
    let device = &context.device;
    let pipeline_cache = &context.pipeline_cache;
    let vert_shader_code = include_aligned!(Align32, "./shaders/shader.vert.spv");
    let frag_shader_code = include_aligned!(Align32, "./shaders/shader.frag.spv");
    let vertex_shader_module = create_shader_module(device, vert_shader_code);
    let frag_shader_module = create_shader_module(device, frag_shader_code);
    let name = CString::new("main").unwrap();
    let vertex_shader_stage_info = vk::PipelineShaderStageCreateInfo::builder()
        .stage(vk::ShaderStageFlags::VERTEX)
        .module(vertex_shader_module)
        .name(name.as_c_str())
        .build();
    let frag_shader_stage_info = vk::PipelineShaderStageCreateInfo::builder()
        .stage(vk::ShaderStageFlags::FRAGMENT)
        .module(frag_shader_module)
        .name(name.as_c_str())
        .build();
    let shader_stages = [vertex_shader_stage_info, frag_shader_stage_info];
    let vertex_input_info = vk::PipelineVertexInputStateCreateInfo::builder();
    let input_assembly_create_info = vk::PipelineInputAssemblyStateCreateInfo::builder()
        .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
        .primitive_restart_enable(false);
    let viewport_state_create_info = vk::PipelineViewportStateCreateInfo::builder()
        .viewport_count(1)
        .scissor_count(1);
    let rasterizer_create_info = vk::PipelineRasterizationStateCreateInfo::builder()
        .depth_clamp_enable(false)
        .rasterizer_discard_enable(false)
        .polygon_mode(vk::PolygonMode::FILL)
        .line_width(1.0)
        .cull_mode(vk::CullModeFlags::BACK)
        .front_face(vk::FrontFace::CLOCKWISE)
        .depth_bias_enable(false);
    let multisampling_create_info = vk::PipelineMultisampleStateCreateInfo::builder()
        .sample_shading_enable(false)
        .rasterization_samples(vk::SampleCountFlags::TYPE_1)
        .min_sample_shading(1.0);
    let front = vk::StencilOpState::builder()
        .fail_op(vk::StencilOp::KEEP)
        .pass_op(vk::StencilOp::KEEP)
        .depth_fail_op(vk::StencilOp::KEEP)
        .compare_op(vk::CompareOp::ALWAYS)
        .build();
    let back = vk::StencilOpState::builder()
        .fail_op(vk::StencilOp::KEEP)
        .pass_op(vk::StencilOp::KEEP)
        .depth_fail_op(vk::StencilOp::KEEP)
        .compare_op(vk::CompareOp::ALWAYS)
        .build();
    let depth_stencil_state = vk::PipelineDepthStencilStateCreateInfo::builder()
        .depth_test_enable(true)
        .depth_write_enable(true)
        .depth_compare_op(vk::CompareOp::LESS_OR_EQUAL)
        .front(front)
        .back(back)
        .min_depth_bounds(0.0)
        .max_depth_bounds(1.0)
        .build();
    let color_blend_attachment = vk::PipelineColorBlendAttachmentState::builder()
        .blend_enable(false)
        .src_color_blend_factor(vk::BlendFactor::ONE)
        .dst_color_blend_factor(vk::BlendFactor::ZERO)
        .color_blend_op(vk::BlendOp::ADD)
        .src_alpha_blend_factor(vk::BlendFactor::ONE)
        .dst_alpha_blend_factor(vk::BlendFactor::ZERO)
        .alpha_blend_op(vk::BlendOp::ADD)
        .color_write_mask(
            vk::ColorComponentFlags::R | vk::ColorComponentFlags::G | vk::ColorComponentFlags::B,
        )
        .build();
    let color_blend_attachments = [color_blend_attachment];
    let blend_constants = [0.0, 0.0, 0.0, 0.0];
    let color_blend_state = vk::PipelineColorBlendStateCreateInfo::builder()
        .logic_op_enable(false)
        .logic_op(vk::LogicOp::CLEAR)
        .blend_constants(blend_constants)
        .attachments(&color_blend_attachments);
    let dynamic_states = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
    let dynamic_pipeline_state_create_info = vk::PipelineDynamicStateCreateInfo::builder()
        .dynamic_states(&dynamic_states)
        .build();
    let pipeline_layout_create_info = vk::PipelineLayoutCreateInfo::builder();
    let pipeline_layout = unsafe {
        device
            .create_pipeline_layout(&pipeline_layout_create_info, None)
            .unwrap()
    };
    let pipeline_create_info = vk::GraphicsPipelineCreateInfo::builder()
        .stages(&shader_stages)
        .vertex_input_state(&vertex_input_info)
        .input_assembly_state(&input_assembly_create_info)
        .viewport_state(&viewport_state_create_info)
        .rasterization_state(&rasterizer_create_info)
        .multisample_state(&multisampling_create_info)
        .depth_stencil_state(&depth_stencil_state)
        .color_blend_state(&color_blend_state)
        .dynamic_state(&dynamic_pipeline_state_create_info)
        .layout(pipeline_layout)
        .render_pass(render_pass)
        .subpass(0)
        .build();
    let create_infos = [pipeline_create_info];
    let mut graphics_pipelines = unsafe {
        device
            .create_graphics_pipelines(*pipeline_cache, &create_infos, None)
            .expect("Unable to get graphics pipeline!")
    };
    // unsafe { device.destroy_shader_module(vertex_shader_module, None) };
    // unsafe { device.destroy_shader_module(frag_shader_module, None) };
    return graphics_pipelines
        .pop()
        .expect("Unable to get graphics pipeline!");
}

pub fn create_shader_module(device: &Device, bytes: &[u8]) -> vk::ShaderModule {
    let code = bytes.as_slice_of().unwrap();
    let create_info = vk::ShaderModuleCreateInfo::builder().code(code);

    unsafe {
        device
            .create_shader_module(&create_info, None)
            .expect("Unable to create shader module")
    }
}
