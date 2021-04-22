use ash::vk;
use ovr_mobile_sys::ovrTextureSwapChain;

use crate::vulkan_context::VulkanContext;
use crate::{texture_swap_chain::TextureSwapChain, texture::Texture};

pub struct FrameBuffer {
    pub width: i32,
    pub height: i32,
    pub swapchain_handle: ovrTextureSwapChain,
    pub texture_swap_chain_length: i32,
    pub colour_textures: Vec<Texture>,
    pub fragment_density_textures: Vec<Texture>,
    pub framebuffers: Vec<vk::Framebuffer>,
    pub num_layers: usize,
    pub current_buffer: usize,
    pub current_layer: usize,
}

impl FrameBuffer {
    pub fn new(
        color_swap_chain: &TextureSwapChain,
        context: &VulkanContext,
        width: i32,
        height: i32,
    ) -> Self {
        let texture_swap_chain_length = color_swap_chain.length;
        let format = color_swap_chain.format;
        let colour_textures = color_swap_chain
            .images
            .iter()
            .map(|i| Texture::new(width, height, format, i, context))
            .collect::<Vec<_>>();

        let mut fragment_density_textures = Vec::new();
        let mut framebuffers = Vec::new();

        Self {
            width,
            height,
            swapchain_handle: color_swap_chain.handle,
            texture_swap_chain_length,
            colour_textures,
            fragment_density_textures,
            framebuffers,
            num_layers: 2,
            current_buffer: 0,
            current_layer: 0,
        }
    }
}
