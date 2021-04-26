use crate::{
    eye_command_buffer::EyeCommandBuffer,
    old_vulkan::{create_graphics_pipeline, SyncObjects},
};
use crate::{
    eye_frame_buffer::EyeFrameBuffer,
    eye_texture_swap_chain::EyeTextureSwapChain,
    old_vulkan::{create_command_buffers, create_sync_objects},
    render_pass::RenderPass,
    vulkan_context::VulkanContext,
};

use ash::{version::DeviceV1_0, vk};
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

pub struct VulkanRenderer {
    pub context: VulkanContext,
    pub current_frame: usize,
    pub render_pass: RenderPass,
    pub eye_command_buffers: [EyeCommandBuffer; 2],
    pub eye_frame_buffers: [EyeFrameBuffer; 2],
    pub sync_objects: [SyncObjects; 2],
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

        let (_, graphics_pipeline) =
            create_graphics_pipeline(&context.device, extent, render_pass.render_pass);

        let eye_command_buffers = [
            EyeCommandBuffer::new(buffers_count, &context),
            EyeCommandBuffer::new(buffers_count, &context),
        ];

        let sync_objects = [
            create_sync_objects(&context.device, buffers_count),
            create_sync_objects(&context.device, buffers_count),
        ];

        println!("[VulkanRenderer] ..done! Renderer initialized");

        Self {
            context,
            current_frame: 0,
            render_pass,
            eye_command_buffers,
            eye_frame_buffers,
            sync_objects,
            extent,
            graphics_pipeline,
        }
    }

    pub unsafe fn render(&mut self, ovr_mobile: NonNull<ovrMobile>) -> () {
        if self.current_frame == 0 {
            self.current_frame += 1;
            return self.render_loading_scene(ovr_mobile);
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
            texture.SwapChainIndex = eye_frame_buffer.current_buffer as i32;
            texture.TexCoordsFromTanAngles =
                ovrMatrix4f_TanAngleMatrixFromProjection(&tracking.Eye[eye].ProjectionMatrix);
        }

        layer.HeadPose = tracking.HeadPose;
        let layers = [&layer.Header as *const ovrLayerHeader2];

        let frame_desc = ovrSubmitFrameDescription2_ {
            Flags: 0,
            FrameIndex: self.current_frame as u64,
            SwapInterval: 1,
            DisplayTime: predicted_display_time,
            LayerCount: layers.len() as u32,
            Layers: layers.as_ptr(),
            Pad: std::mem::zeroed(),
        };

        // Hand over the eye images to the time warp.
        let result = vrapi_SubmitFrame2(ovr_mobile, &frame_desc);
        self.current_frame += 1;
        assert_eq!(0, result);
    }

    pub fn draw_frame(&mut self, eye: usize) {
        let eye_frame_buffers = &self.eye_frame_buffers[eye];
        let eye_command_buffer = &self.eye_command_buffers[eye as usize];

        let current_buffer_index = eye_frame_buffers.current_buffer;
        let current_command_buffer = eye_command_buffer.command_buffers[current_buffer_index];
        let current_frame_buffer = eye_frame_buffers.frame_buffers[current_buffer_index];

        self.write_command_buffer(current_command_buffer, current_frame_buffer);

        let submit_info = vk::SubmitInfo::builder()
            .command_buffers(&[current_command_buffer])
            .build();

        let submits = [submit_info];

        unsafe {
            self.context
                .device
                .queue_submit(self.context.graphics_queue, &submits, vk::Fence::null())
                .expect("Unable to submit to queue")
        };

        self.eye_frame_buffers[eye].current_buffer = (current_buffer_index + 1) % 3;
    }

    pub fn write_command_buffer(
        &mut self,
        command_buffer: vk::CommandBuffer,
        frame_buffer: vk::Framebuffer,
    ) {
        let extent = self.extent;
        let device = &self.context.device;
        let begin_info = vk::CommandBufferBeginInfo::builder()
            .flags(vk::CommandBufferUsageFlags::SIMULTANEOUS_USE);
        let render_pass = self.render_pass.render_pass;
        let pipeline = self.graphics_pipeline;

        unsafe {
            device
                .begin_command_buffer(command_buffer, &begin_info)
                .expect("Unable to begin command buffer");
        }
        let render_area = vk::Rect2D {
            offset: vk::Offset2D { x: 0, y: 0 },
            extent,
        };

        let clear_color = vk::ClearValue {
            color: vk::ClearColorValue {
                float32: [0.0, 0.0, 0.0, 0.0],
            },
        };

        let clear_colors = [clear_color];

        let render_pass_info = vk::RenderPassBeginInfo::builder()
            .render_pass(render_pass)
            .framebuffer(frame_buffer)
            .render_area(render_area)
            .clear_values(&clear_colors);

        unsafe {
            device.cmd_begin_render_pass(
                command_buffer,
                &render_pass_info,
                vk::SubpassContents::INLINE,
            );
            device.cmd_bind_pipeline(command_buffer, vk::PipelineBindPoint::GRAPHICS, pipeline);
            device.cmd_draw(command_buffer, 3, 1, 0, 0);
            device.cmd_end_render_pass(command_buffer);
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
