use crate::{
    eye_command_buffer::EyeCommandBuffer, eye_frame_buffer::EyeFrameBuffer,
    eye_texture_swap_chain::EyeTextureSwapChain, render_pass::RenderPass, texture::Texture,
    vulkan_context::VulkanContext,
};
use std::ffi::CString;

use ash::{version::DeviceV1_0, vk, Device};
use byte_slice_cast::AsSliceOf;
use ovr_mobile_sys::{
    helpers::{
        ovrMatrix4f_TanAngleMatrixFromProjection, vrapi_DefaultLayerBlackProjection2,
        vrapi_DefaultLayerLoadingIcon2, vrapi_DefaultLayerProjection2,
    },
    ovrFrameFlags_::VRAPI_FRAME_FLAG_FLUSH,
    ovrFrameLayerFlags_::VRAPI_FRAME_LAYER_FLAG_INHIBIT_SRGB_FRAMEBUFFER,
    ovrJava, ovrLayerHeader2, ovrMobile, ovrSubmitFrameDescription2_,
    ovrSystemProperty_::{
        VRAPI_SYS_PROP_SUGGESTED_EYE_TEXTURE_HEIGHT, VRAPI_SYS_PROP_SUGGESTED_EYE_TEXTURE_WIDTH,
    },
    vrapi_GetPredictedDisplayTime, vrapi_GetPredictedTracking2, vrapi_GetSystemPropertyInt,
    vrapi_SubmitFrame2,
};
use std::ptr::NonNull;

pub const COLOUR_FORMAT: vk::Format = vk::Format::R8G8B8A8_UNORM;
pub const DEPTH_FORMAT: vk::Format = vk::Format::D24_UNORM_S8_UINT;

pub struct VulkanRenderer {
    pub context: VulkanContext,
    pub current_frame: usize,
    pub render_pass: RenderPass,
    pub eye_command_buffers: [EyeCommandBuffer; 2],
    pub eye_frame_buffers: [EyeFrameBuffer; 2],
    // pub sync_objects: [SyncObjects; 2],
    pub extent: vk::Extent2D,
    pub graphics_pipeline: vk::Pipeline,
}

impl VulkanRenderer {
    pub unsafe fn new(java: &ovrJava) -> Self {
        println!("[VulkanRenderer] Initialising renderer..");
        let context = VulkanContext::new();
        let buffers_count = 3;
        let width = vrapi_GetSystemPropertyInt(java, VRAPI_SYS_PROP_SUGGESTED_EYE_TEXTURE_WIDTH);
        let height = vrapi_GetSystemPropertyInt(java, VRAPI_SYS_PROP_SUGGESTED_EYE_TEXTURE_HEIGHT);
        let extent = vk::Extent2D {
            width: width as u32,
            height: height as u32,
        };

        let eye_texture_swap_chains = [
            EyeTextureSwapChain::new(width, height), // left eye
            EyeTextureSwapChain::new(width, height), // right eye
        ];

        let render_pass = RenderPass::new(&context.device);
        let eye_frame_buffers = [
            EyeFrameBuffer::new(
                &eye_texture_swap_chains[0],
                &render_pass,
                &context,
                width,
                height,
            ),
            EyeFrameBuffer::new(
                &eye_texture_swap_chains[1],
                &render_pass,
                &context,
                width,
                height,
            ),
        ];

        let graphics_pipeline = create_graphics_pipeline(&context, &render_pass);

        let eye_command_buffers = [
            EyeCommandBuffer::new(buffers_count, &context),
            EyeCommandBuffer::new(buffers_count, &context),
        ];

        // let sync_objects = [
        //     create_sync_objects(&context.device, buffers_count),
        //     create_sync_objects(&context.device, buffers_count),
        // ];

        println!("[VulkanRenderer] ..done! Renderer initialized");

        Self {
            context,
            current_frame: 0,
            render_pass,
            eye_command_buffers,
            eye_frame_buffers,
            // sync_objects,
            extent,
            graphics_pipeline,
        }
    }

    pub unsafe fn render(&mut self, ovr_mobile: NonNull<ovrMobile>) -> () {
        if self.current_frame == 0 {
            self.render_loading_scene(ovr_mobile);
        }

        self.current_frame += 1;
        for eye in 0..2 {
            let current_buffer_index = self.eye_frame_buffers[eye].current_buffer_index;
            self.eye_frame_buffers[eye].current_buffer_index = (current_buffer_index + 1) % 3;
        }

        let ovr_mobile = ovr_mobile.as_ptr();
        let predicted_display_time =
            vrapi_GetPredictedDisplayTime(ovr_mobile, self.current_frame as i64);
        let tracking = vrapi_GetPredictedTracking2(ovr_mobile, predicted_display_time);
        let mut layer = vrapi_DefaultLayerProjection2();

        for eye in 0..2 {
            self.draw_frame(eye);
            let mut texture = layer.Textures[eye];
            let eye_frame_buffer = &self.eye_frame_buffers[eye];
            texture.ColorSwapChain = eye_frame_buffer.swapchain_handle.as_ptr();
            texture.SwapChainIndex = eye_frame_buffer.current_buffer_index as i32;
            texture.TexCoordsFromTanAngles =
                ovrMatrix4f_TanAngleMatrixFromProjection(&tracking.Eye[eye].ProjectionMatrix);

            // println!("[Renderer] {} - {:?}", predicted_display_time, texture);
        }

        layer.HeadPose = tracking.HeadPose;
        let layers = [&layer.Header as *const ovrLayerHeader2];

        let frame_desc = ovrSubmitFrameDescription2_ {
            Flags: 0,
            FrameIndex: self.current_frame as u64,
            SwapInterval: 1,
            DisplayTime: predicted_display_time,
            LayerCount: 1,
            Layers: layers.as_ptr(),
            Pad: std::mem::zeroed(),
        };

        // Hand over the eye images to the time warp.
        let result = vrapi_SubmitFrame2(ovr_mobile, &frame_desc);
        assert_eq!(0, result);
    }

    pub fn draw_frame(&mut self, eye: usize) {
        {
            let eye_frame_buffers = &self.eye_frame_buffers[eye];
            let current_buffer_index = eye_frame_buffers.current_buffer_index;
            self.wait_for_fence(eye, current_buffer_index);
        }

        let eye_frame_buffers = &self.eye_frame_buffers[eye];
        let current_buffer_index = eye_frame_buffers.current_buffer_index;
        let eye_command_buffer = &mut self.eye_command_buffers[eye as usize];

        let current_command_buffer = eye_command_buffer.command_buffers[current_buffer_index];
        let current_frame_buffer = eye_frame_buffers.frame_buffers[current_buffer_index];
        let current_texture = &eye_frame_buffers.display_textures[current_buffer_index];

        {
            self.write_command_buffer(
                current_texture,
                current_command_buffer,
                current_frame_buffer,
            );
        }

        let eye_command_buffer = &mut self.eye_command_buffers[eye as usize];
        let fence = &mut eye_command_buffer.fences[current_buffer_index];

        let submit_info = vk::SubmitInfo::builder()
            .command_buffers(&[current_command_buffer])
            .build();

        let submits = [submit_info];

        unsafe {
            self.context
                .device
                .queue_submit(self.context.graphics_queue, &submits, fence.fence)
                .expect("Unable to submit to queue");
        };

        fence.submitted = true;
    }

    fn wait_for_fence(&mut self, eye: usize, current_buffer_index: usize) {
        let eye_command_buffer = &mut self.eye_command_buffers[eye as usize];
        let fence = &mut eye_command_buffer.fences[current_buffer_index];
        if fence.submitted {
            unsafe {
                self.context
                    .device
                    .wait_for_fences(&[fence.fence], true, u64::MAX)
                    .expect("Unable to wait for fence");
                self.context
                    .device
                    .reset_fences(&[fence.fence])
                    .expect("Unable to reset fence");
                fence.submitted = false;
            };
        }
    }

    pub fn write_command_buffer(
        &self,
        texture: &Texture,
        command_buffer: vk::CommandBuffer,
        frame_buffer: vk::Framebuffer,
    ) {
        let extent = self.extent;
        let device = &self.context.device;
        let begin_info = vk::CommandBufferBeginInfo::builder();
        let render_pass = self.render_pass.render_pass;
        let pipeline = self.graphics_pipeline;
        let offset = vk::Offset2D { x: 0, y: 0 };
        let render_area = vk::Rect2D { offset, extent };
        let clear_color = vk::ClearValue {
            color: vk::ClearColorValue {
                float32: [0.125, 0.0, 0.125, 1.0],
            },
        };
        let depth_value = vk::ClearValue {
            depth_stencil: vk::ClearDepthStencilValue {
                depth: 1.0,
                stencil: 0,
            },
        };
        let clear_colors = [clear_color, depth_value];
        let render_pass_info = vk::RenderPassBeginInfo::builder()
            .render_pass(render_pass)
            .framebuffer(frame_buffer)
            .render_area(render_area)
            .clear_values(&clear_colors);
        let viewport = vk::Viewport::builder()
            .x(0.0)
            .y(0.0)
            .min_depth(0.0)
            .max_depth(1.0)
            .width(extent.width as f32)
            .height(extent.height as f32)
            .build();
        let scissor = vk::Rect2D::builder().extent(extent).offset(offset).build();

        let begin_flags = vk::AccessFlags::SHADER_READ;
        let end_flags =
            vk::AccessFlags::COLOR_ATTACHMENT_READ | vk::AccessFlags::COLOR_ATTACHMENT_WRITE;
        let begin_layout = vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL;
        let end_layout = vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL;
        let begin_stage =
            vk::PipelineStageFlags::VERTEX_SHADER | vk::PipelineStageFlags::FRAGMENT_SHADER;
        let end_stage = vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT;

        unsafe {
            device
                .reset_command_buffer(command_buffer, vk::CommandBufferResetFlags::empty())
                .expect("Unable to reset command buffer");
            device
                .begin_command_buffer(command_buffer, &begin_info)
                .expect("Unable to begin command buffer");
        }

        self.context.change_image_layout(
            command_buffer,
            &texture.image,
            begin_flags,
            end_flags,
            begin_layout,
            end_layout,
            begin_stage,
            end_stage,
        );

        unsafe {
            device.cmd_begin_render_pass(
                command_buffer,
                &render_pass_info,
                vk::SubpassContents::INLINE,
            );
            device.cmd_set_viewport(command_buffer, 0, &[viewport]);
            device.cmd_set_scissor(command_buffer, 0, &[scissor]);
            device.cmd_bind_pipeline(command_buffer, vk::PipelineBindPoint::GRAPHICS, pipeline);
            device.cmd_draw(command_buffer, 3, 1, 0, 0);
            device.cmd_end_render_pass(command_buffer);
        }

        self.context.change_image_layout(
            command_buffer,
            &texture.image,
            end_flags,
            begin_flags,
            end_layout,
            begin_layout,
            end_stage,
            begin_stage,
        );

        unsafe {
            device
                .end_command_buffer(command_buffer)
                .expect("Unable to record command buffer!");
        }
    }

    pub unsafe fn render_loading_scene(&mut self, ovr_mobile: NonNull<ovrMobile>) -> () {
        println!("[Renderer] Rendering loading scene..");
        let ovr_mobile = ovr_mobile.as_ptr();

        let predicted_display_time =
            vrapi_GetPredictedDisplayTime(ovr_mobile, self.current_frame as i64);
        let _tracking = vrapi_GetPredictedTracking2(ovr_mobile, predicted_display_time);
        let mut blackLayer = vrapi_DefaultLayerBlackProjection2();
        blackLayer.Header.Flags |= VRAPI_FRAME_LAYER_FLAG_INHIBIT_SRGB_FRAMEBUFFER as u32;

        let mut iconLayer = vrapi_DefaultLayerLoadingIcon2();
        iconLayer.Header.Flags |= VRAPI_FRAME_LAYER_FLAG_INHIBIT_SRGB_FRAMEBUFFER as u32;

        let layers = [
            &blackLayer.Header as *const ovrLayerHeader2,
            &iconLayer.Header as *const ovrLayerHeader2,
        ];

        let mut frameFlags = 0;
        frameFlags |= VRAPI_FRAME_FLAG_FLUSH as u32;

        let frame_desc = ovrSubmitFrameDescription2_ {
            Flags: frameFlags,
            FrameIndex: 0,
            SwapInterval: 1,
            DisplayTime: predicted_display_time,
            LayerCount: layers.len() as u32,
            Layers: layers.as_ptr(),
            Pad: std::mem::zeroed(),
        };

        // Hand over the eye images to the time warp.
        let result = vrapi_SubmitFrame2(ovr_mobile, &frame_desc);
        assert_eq!(0, result);
        println!("[Renderer] ..done, now rendering first real frames.");
    }
}

fn create_graphics_pipeline(context: &VulkanContext, render_pass: &RenderPass) -> vk::Pipeline {
    let device = &context.device;
    let pipeline_cache = &context.pipeline_cache;
    let render_pass = render_pass.render_pass;
    let vert_shader_code = include_bytes!("./shaders/shader.vert.spv");
    let frag_shader_code = include_bytes!("./shaders/shader.frag.spv");
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
    unsafe { device.destroy_shader_module(vertex_shader_module, None) };
    unsafe { device.destroy_shader_module(frag_shader_module, None) };
    return graphics_pipelines
        .pop()
        .expect("Unable to get graphics pipeline!");
}

fn create_shader_module(device: &Device, bytes: &[u8]) -> vk::ShaderModule {
    // let (_, code, _) = unsafe { bytes.align_to::<u32>() };
    let create_info = vk::ShaderModuleCreateInfo::builder().code(bytes.as_slice_of().unwrap());

    unsafe {
        device
            .create_shader_module(&create_info, None)
            .expect("Unable to create shader module")
    }
}
