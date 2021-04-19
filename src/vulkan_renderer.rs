use ovr_mobile_sys::ovrMatrix4f;

use crate::{
    eye_command_buffer::EyeCommandBuffer, frame_buffer::FrameBuffer, render_pass::RenderPass,
};

// typedef struct {
//     ovrVkRenderPass RenderPassSingleView;
//     ovrVkCommandBuffer EyeCommandBuffer[VRAPI_FRAME_LAYER_EYE_MAX];
//     ovrFrameBuffer Framebuffer[VRAPI_FRAME_LAYER_EYE_MAX];

//     ovrMatrix4f ViewMatrix[VRAPI_FRAME_LAYER_EYE_MAX];
//     ovrMatrix4f ProjectionMatrix[VRAPI_FRAME_LAYER_EYE_MAX];
//     int NumEyes;
// } ovrRenderer;

pub struct VulkanRenderer {
    pub render_pass_single_view: RenderPass,
    pub eye_command_buffers: Vec<EyeCommandBuffer>,
    pub frame_buffers: Vec<FrameBuffer>,
    pub view_matrix: Vec<ovrMatrix4f>,
    pub projection_matrix: Vec<ovrMatrix4f>,
}
