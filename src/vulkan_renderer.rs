use crate::{
    frame_buffer::FrameBuffer, render_pass::RenderPass, texture_swap_chain::TextureSwapChain,
    vulkan_context::VulkanContext,
};
use ash::vk;
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
    pub eye_command_buffers: Vec<vk::CommandBuffer>,
    pub frame_buffers: Vec<FrameBuffer>,
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

        let texture_swap_chains = (0..eyes)
            .map(|_| TextureSwapChain::new(width, height))
            .collect::<Vec<_>>();

        let render_pass = RenderPass::new(&context.device);
        let frame_buffers = texture_swap_chains
            .iter()
            .map(|t| FrameBuffer::new(t, &render_pass, &context, width, height))
            .collect::<Vec<_>>();

        let eye_command_buffers = frame_buffers
            .iter()
            .map(|frame_buffer| create_eye_command_buffer(frame_buffer, &context))
            .collect::<Vec<_>>();

        println!("[VulkanRenderer] ..done! Renderer initialized");

        Self {
            context,
            frame_index: 0,
            render_pass,
            eye_command_buffers,
            frame_buffers,
            // view_matrix,
            // projection_matrix,
        }
    }

    pub unsafe fn render(&mut self, ovr_mobile: NonNull<ovrMobile>) -> () {
        let ovr_mobile = ovr_mobile.as_ptr();
        println!("[RENDER] In render..");
        // Get the HMD pose, predicted for the middle of the time period during which
        // the new eye images will be displayed. The number of frames predicted ahead
        // depends on the pipeline depth of the engine and the synthesis rate.
        // The better the prediction, the less black will be pulled in at the edges.
        let predicted_display_time = vrapi_GetPredictedDisplayTime(ovr_mobile, self.frame_index);
        let _tracking = vrapi_GetPredictedTracking2(ovr_mobile, predicted_display_time);

        // Advance the simulation based on the predicted display time.

        // Render eye images and setup the 'ovrSubmitFrameDescription2' using 'ovrTracking2' data.

        let mut blackLayer = vrapi_DefaultLayerBlackProjection2();
        blackLayer.Header.Flags |= VRAPI_FRAME_LAYER_FLAG_INHIBIT_SRGB_FRAMEBUFFER as u32;

        let mut iconLayer = vrapi_DefaultLayerLoadingIcon2();
        iconLayer.Header.Flags |= VRAPI_FRAME_LAYER_FLAG_INHIBIT_SRGB_FRAMEBUFFER as u32;
        // layer.HeadPose = tracking.HeadPose;
        // for eye in 0..2 {
        //     let colorTextureSwapChainIndex = self.frame_index as i32
        //         % vrapi_GetTextureSwapChainLength(self.color_texture_swap_chain[eye]);
        //     let textureId = vrapi_GetTextureSwapChainHandle(
        //         self.color_texture_swap_chain[eye],
        //         colorTextureSwapChainIndex,
        //     );
        //     //     // Render to 'textureId' using the 'ProjectionMatrix' from 'ovrTracking2'.

        //     layer.Textures[eye].ColorSwapChain = self.color_texture_swap_chain[eye];
        //     layer.Textures[eye].SwapChainIndex = colorTextureSwapChainIndex;
        //     layer.Textures[eye].TexCoordsFromTanAngles =
        //         ovrMatrix4f_TanAngleMatrixFromProjection(&tracking.Eye[eye].ProjectionMatrix);
        // }

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

        println!("[RENDER] About to submit frame..");

        // Hand over the eye images to the time warp.
        let result = vrapi_SubmitFrame2(ovr_mobile, &frame_desc);
        println!("[RENDER] Submit frame result: {:?}", result);
    }
}

fn create_eye_command_buffer(
    frame_buffer: &FrameBuffer,
    context: &VulkanContext,
) -> vk::CommandBuffer {
    todo!()
}
