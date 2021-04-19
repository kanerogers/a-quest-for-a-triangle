use ash::{version::InstanceV1_0, vk::Handle, Instance};
use ovr_mobile_sys::{
    ovrMatrix4f, ovrSystemCreateInfoVulkan, vrapi_CreateSystemVulkan, VkInstance_T,
};

use crate::{
    eye_command_buffer::EyeCommandBuffer, frame_buffer::FrameBuffer, render_pass::RenderPass,
};

pub struct VulkanRenderer {
    pub render_pass_single_view: RenderPass,
    pub eye_command_buffers: Vec<EyeCommandBuffer>,
    pub frame_buffers: Vec<FrameBuffer>,
    pub view_matrix: Vec<ovrMatrix4f>,
    pub projection_matrix: Vec<ovrMatrix4f>,
    pub num_eyes: usize,
}

impl VulkanRenderer {
    pub fn new(
        render_pass_single_view: RenderPass,
        eye_command_buffers: Vec<EyeCommandBuffer>,
        frame_buffers: Vec<FrameBuffer>,
        view_matrix: Vec<ovrMatrix4f>,
        projection_matrix: Vec<ovrMatrix4f>,
        num_eyes: usize,
    ) -> Self {
        let instance = create_instance();
        let vk_instance = instance.handle().as_raw();
        let PhysicalDevice = get_physical_device();
        let Device = get_device();

        let mut system_info = ovrSystemCreateInfoVulkan {
            Instance: vk_instance as *mut VkInstance_T,
            PhysicalDevice,
            Device,
        };

        unsafe {
            vrapi_CreateSystemVulkan(&mut system_info);
        }

        Self {
            render_pass_single_view,
            eye_command_buffers,
            frame_buffers,
            view_matrix,
            projection_matrix,
            num_eyes,
        }
    }
}

fn get_physical_device() -> *mut ovr_mobile_sys::VkPhysicalDevice_T {
    todo!()
}

fn get_device() -> *mut ovr_mobile_sys::VkDevice_T {
    todo!()
}

fn create_instance() -> Instance {
    todo!()
}
