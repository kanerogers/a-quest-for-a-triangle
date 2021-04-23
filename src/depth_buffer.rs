use ash::vk;

use crate::vulkan_context::VulkanContext;

#[derive(Debug, Clone, Copy)]
pub struct DepthBuffer {
    pub format: vk::Format,
    pub layout: vk::ImageLayout,
    pub image: vk::Image,
    pub memory: vk::DeviceMemory,
    pub view: vk::ImageView,
}

impl DepthBuffer {
    pub fn new(width: i32, height: i32, format: vk::Format, context: &VulkanContext) -> Self {
        Self {
            format,
            layout: todo!(),
            image: todo!(),
            memory: todo!(),
            view: todo!(),
        }
    }
}
