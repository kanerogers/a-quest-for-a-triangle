use crate::context::Context;

pub struct VulkanRenderer {
    context: Context,
    // pub render_pass_single_view: RenderPass,
    // pub eye_command_buffers: Vec<EyeCommandBuffer>,
    // pub frame_buffers: Vec<FrameBuffer>,
    // pub view_matrix: Vec<ovrMatrix4f>,
    // pub projection_matrix: Vec<ovrMatrix4f>,
    // pub num_eyes: usize,
}

impl VulkanRenderer {
    pub unsafe fn new() -> Self {
        let context = Context::new();
        Self {
            context,
            // render_pass_single_view,
            // eye_command_buffers,
            // frame_buffers,
            // view_matrix,
            // projection_matrix,
            // num_eyes,
        }
    }
}
