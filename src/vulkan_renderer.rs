use crate::old_vulkan::create_graphics_pipeline;
use crate::{
    eye_command_buffer::EyeCommandBuffer,
    eye_frame_buffer::EyeFrameBuffer,
    eye_texture_swap_chain::EyeTextureSwapChain,
    old_vulkan::{create_command_buffers, create_sync_objects, MAX_FRAMES_IN_FLIGHT},
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
    pub eye_command_buffers: [Vec<vk::CommandBuffer>; 2],
    pub eye_frame_buffers: [EyeFrameBuffer; 2],
    pub in_flight_fences: Vec<vk::Fence>,
    pub image_available_semaphores: Vec<vk::Semaphore>,
    pub render_finished_semaphores: Vec<vk::Semaphore>,
}

impl VulkanRenderer {
    pub unsafe fn new(java: &ovrJava) -> Self {
        println!("[VulkanRenderer] Initialising renderer..");
        let context = VulkanContext::new();
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
            create_command_buffers(
                &context.device,
                &eye_frame_buffers[0].frame_buffers,
                context.command_pool,
                render_pass.render_pass,
                extent,
                graphics_pipeline,
            ),
            create_command_buffers(
                &context.device,
                &eye_frame_buffers[1].frame_buffers,
                context.command_pool,
                render_pass.render_pass,
                extent,
                graphics_pipeline,
            ),
        ];

        let (
            image_available_semaphores,
            render_finished_semaphores,
            in_flight_fences,
            images_in_flight,
        ) = create_sync_objects(&context.device, 2);

        println!("[VulkanRenderer] ..done! Renderer initialized");

        Self {
            context,
            current_frame: 0,
            render_pass,
            eye_command_buffers,
            eye_frame_buffers,
            in_flight_fences,
            image_available_semaphores,
            render_finished_semaphores,
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
        let fence = self
            .in_flight_fences
            .get(self.current_frame)
            .expect("Unable to get fence!");
        let fences = [*fence];

        unsafe { self.context.device.wait_for_fences(&fences, true, u64::MAX) }
            .expect("Unable to wait for fence");

        let image_available_semaphore = self
            .image_available_semaphores
            .get(self.current_frame)
            .expect("Unable to get image_available semaphore for frame!");
        let render_finished_semaphore = self
            .render_finished_semaphores
            .get(self.current_frame)
            .expect("Unable to get render_finished semaphore");

        // let wait_semaphores = [*image_available_semaphore];
        let wait_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];

        let eye_command_buffers = self.eye_command_buffers.get(eye as usize).unwrap();

        let signal_semaphores = [*render_finished_semaphore];

        let submit_info = vk::SubmitInfo::builder()
            .command_buffers(&eye_command_buffers)
            // .wait_semaphores(&wait_semaphores)
            .wait_dst_stage_mask(&wait_stages)
            .signal_semaphores(&signal_semaphores)
            .build();

        let submits = [submit_info];

        // self.images_in_flight[image_index as usize] = None;
        unsafe { self.context.device.reset_fences(&fences) }.expect("Unable to reset fences");
        unsafe {
            self.context
                .device
                .queue_submit(self.context.graphics_queue, &submits, *fence)
                .expect("Unable to submit to queue")
        };
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
    }
}
