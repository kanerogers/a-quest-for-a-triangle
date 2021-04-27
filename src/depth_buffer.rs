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
        let usage = vk::ImageUsageFlags::TRANSIENT_ATTACHMENT
            | vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT;
        let image = context._create_image(width, height, format, usage);
        let aspect_mask = vk::ImageAspectFlags::DEPTH | vk::ImageAspectFlags::STENCIL;
        let view = context.create_image_view(&image, format, aspect_mask);

        let src_access_mask = vk::AccessFlags::empty();
        let dst_access_mask = vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE;
        let old_layout = vk::ImageLayout::UNDEFINED;
        let new_layout = vk::ImageLayout::DEPTH_ATTACHMENT_OPTIMAL;
        let start_stage = vk::PipelineStageFlags::TOP_OF_PIPE;
        let end_stage = vk::PipelineStageFlags::ALL_GRAPHICS;

        let setup_command_buffer = context.create_setup_command_buffer();
        context.change_image_layout(
            setup_command_buffer,
            &image,
            src_access_mask,
            dst_access_mask,
            old_layout,
            new_layout,
            start_stage,
            end_stage,
        );
        context.flush_setup_command_buffer(setup_command_buffer);

        Self {
            format,
            layout: new_layout,
            image,
            memory: vk::DeviceMemory::null(), // TODO
            view,
        }
    }
}
