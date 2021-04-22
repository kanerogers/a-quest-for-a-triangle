use ash::vk;
use ovr_mobile_sys::ovrTextureSwapChain;

use crate::vulkan_context::VulkanContext;
use crate::{texture::Texture, texture_swap_chain::TextureSwapChain};

pub struct FrameBuffer {
    pub width: i32,
    pub height: i32,
    pub swapchain_handle: ovrTextureSwapChain,
    pub swap_chain_length: i32,
    pub render_textures: Vec<Texture>, // textures that will be submitted for rendering.
    pub ffr_textures: Vec<Texture>,    // textures used for ffr
    pub framebuffers: Vec<vk::Framebuffer>,
    pub num_layers: usize,
    pub current_buffer: usize,
    pub current_layer: usize,
}

impl FrameBuffer {
    pub fn new(
        texture_swap_chain: &TextureSwapChain,
        context: &VulkanContext,
        width: i32,
        height: i32,
    ) -> Self {
        let texture_swap_chain_length = texture_swap_chain.length;
        let format = texture_swap_chain.format;
        let render_textures = texture_swap_chain
            .render_images
            .iter()
            .map(|image| Texture::new(width, height, format, image, context))
            .collect::<Vec<_>>();

        let mut fragment_density_textures = Vec::new();
        let mut framebuffers = Vec::new();

        Self {
            width,
            height,
            swapchain_handle: texture_swap_chain.handle,
            swap_chain_length: texture_swap_chain_length,
            render_textures,
            ffr_textures: fragment_density_textures,
            framebuffers,
            num_layers: 2,
            current_buffer: 0,
            current_layer: 0,
        }
    }
}
