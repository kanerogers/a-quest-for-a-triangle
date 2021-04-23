use ash::{version::DeviceV1_0, vk};

use crate::vulkan_context::VulkanContext;

pub struct EyeCommandBuffer {
    pub num_buffers: usize,
    pub current_buffer: usize,
    pub command_buffers: Vec<vk::CommandBuffer>,
    pub fences: Vec<vk::Fence>,
}

impl EyeCommandBuffer {
    pub fn new(num_buffers: usize, context: &VulkanContext) -> Self {
        println!("[EyeCommandBuffer] Creating eye command buffer..");
        let command_buffers = (0..num_buffers)
            .map(|_| create_command_buffer(context))
            .collect::<Vec<_>>();
        let fences = (0..num_buffers)
            .map(|_| create_fence(context))
            .collect::<Vec<_>>();

        println!("[EyeCommandBuffer] ..done");
        Self {
            current_buffer: 0,
            num_buffers,
            command_buffers,
            fences,
        }
    }
}

fn create_fence(context: &VulkanContext) -> vk::Fence {
    let create_info = vk::FenceCreateInfo::builder();
    unsafe {
        context
            .device
            .create_fence(&create_info, None)
            .expect("Unable to create fence")
    }
}

fn create_command_buffer(context: &VulkanContext) -> vk::CommandBuffer {
    let create_info = vk::CommandBufferAllocateInfo::builder()
        .command_buffer_count(1)
        .command_pool(context.command_pool)
        .level(vk::CommandBufferLevel::PRIMARY);
    unsafe {
        context
            .device
            .allocate_command_buffers(&create_info)
            .expect("Unable to create command_buffers")
            .pop()
            .unwrap()
    }
}
