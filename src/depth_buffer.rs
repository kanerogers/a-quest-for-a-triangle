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
        let usage =
            vk::ImageUsageFlags::TRANSIENT_ATTACHMENT | vk::ImageUsageFlags::INPUT_ATTACHMENT;
        let image = context.create_image(width, height, format, usage);
        let aspect_mask = vk::ImageAspectFlags::DEPTH | vk::ImageAspectFlags::STENCIL;
        let view = context.create_image_view(&image, format, aspect_mask);

        let new_layout = vk::ImageLayout::DEPTH_ATTACHMENT_OPTIMAL;
        let dst_access_mask = vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE;
        context.change_image_layout(&image, dst_access_mask, aspect_mask, new_layout);

        Self {
            format,
            layout: new_layout,
            image,
            memory: vk::DeviceMemory::null(), // TODO
            view,
        }
    }
}
