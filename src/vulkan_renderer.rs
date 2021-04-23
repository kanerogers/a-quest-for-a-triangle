use crate::{
    eye_command_buffer::EyeCommandBuffer, eye_frame_buffer::EyeFrameBuffer,
    eye_texture_swap_chain::EyeTextureSwapChain, render_pass::RenderPass,
    vulkan_context::VulkanContext,
};
use ovr_mobile_sys::{
    helpers::{vrapi_DefaultLayerBlackProjection2, vrapi_DefaultLayerLoadingIcon2},
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
    pub frame_index: i64,
    pub render_pass: RenderPass,
    pub eye_command_buffers: Vec<EyeCommandBuffer>,
    pub eye_frame_buffers: Vec<EyeFrameBuffer>,
    // pub view_matrix: Vec<ovrMatrix4f>,
    // pub projection_matrix: Vec<ovrMatrix4f>,
}

impl VulkanRenderer {
    pub unsafe fn new(java: &ovrJava) -> Self {
        println!("[VulkanRenderer] Initialising renderer..");
        let context = VulkanContext::new();
        let width = vrapi_GetSystemPropertyInt(java, VRAPI_SYS_PROP_SUGGESTED_EYE_TEXTURE_WIDTH);
        let height = vrapi_GetSystemPropertyInt(java, VRAPI_SYS_PROP_SUGGESTED_EYE_TEXTURE_HEIGHT);
        let eyes = 2;

        let eye_texture_swap_chains = (0..eyes)
            .map(|_| EyeTextureSwapChain::new(width, height))
            .collect::<Vec<_>>();

        let render_pass = RenderPass::new(&context.device);
        let frame_buffers = eye_texture_swap_chains
            .iter()
            .map(|t| EyeFrameBuffer::new(t, &render_pass, &context, width, height))
            .collect::<Vec<_>>();

        let eye_command_buffers = frame_buffers
            .iter()
            .map(|f| EyeCommandBuffer::new(f.frame_buffers.len(), &context))
            .collect::<Vec<_>>();

        println!("[VulkanRenderer] ..done! Renderer initialized");

        Self {
            context,
            frame_index: 0,
            render_pass,
            eye_command_buffers,
            eye_frame_buffers: frame_buffers,
            // view_matrix,
            // projection_matrix,
        }
    }

    pub unsafe fn render(&mut self, ovr_mobile: NonNull<ovrMobile>) -> () {
        if self.frame_index == 0 {
            self.frame_index += 1;
            return self.render_loading_scene(ovr_mobile);
        }
    }

    pub unsafe fn render_loading_scene(&mut self, ovr_mobile: NonNull<ovrMobile>) -> () {
        let ovr_mobile = ovr_mobile.as_ptr();

        let predicted_display_time = vrapi_GetPredictedDisplayTime(ovr_mobile, self.frame_index);
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
            FrameIndex: self.frame_index as u64,
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
