use ash::{version::DeviceV1_0, vk};
use ovr_mobile_sys::ovrTextureSwapChain;
use std::ptr::NonNull;

use crate::{
    depth_buffer::DepthBuffer,
    eye_texture_swap_chain::EyeTextureSwapChain,
    render_pass::RenderPass,
    texture::{Texture, TextureUsageFlags},
    vulkan_context::VulkanContext,
};

pub struct EyeFrameBuffer {
    pub width: i32,
    pub height: i32,
    pub swapchain_handle: NonNull<ovrTextureSwapChain>,
    pub swap_chain_length: i32,
    pub display_textures: Vec<Texture>, // textures that will be displayed to the user's eyes
    pub frame_buffers: Vec<vk::Framebuffer>, // ??
    pub current_buffer_index: usize,
}

impl EyeFrameBuffer {
    pub fn new(
        eye_texture_swap_chain: &EyeTextureSwapChain,
        render_pass: &RenderPass,
        context: &VulkanContext,
        width: i32,
        height: i32,
    ) -> Self {
        println!("[EyeFrameBuffer] Creating FrameBuffer..");
        let eye_texture_swap_chain_length = eye_texture_swap_chain.length;
        let display_textures = eye_texture_swap_chain
            .display_images
            .iter()
            .map(|image| Texture::new(width, height, image, context))
            .collect::<Vec<_>>();

        let depth_buffer = DepthBuffer::new(width, height, context);

        let frame_buffers = display_textures
            .iter()
            .map(|t| create_frame_buffer(t, depth_buffer.view, render_pass, context))
            .collect::<Vec<_>>();

        let swapchain_handle = eye_texture_swap_chain.handle;
        println!("[EyeFrameBuffer] Done!");

        Self {
            width,
            height,
            swapchain_handle,
            swap_chain_length: eye_texture_swap_chain_length,
            display_textures,
            frame_buffers,
            current_buffer_index: 0,
        }
    }
}

fn create_frame_buffer(
    texture: &Texture,
    depth_buffer_view: vk::ImageView,
    render_pass: &RenderPass,
    context: &VulkanContext,
) -> vk::Framebuffer {
    let attachments = [texture.view, depth_buffer_view];
    let create_info = vk::FramebufferCreateInfo::builder()
        .attachments(&attachments)
        .width(texture.width as u32)
        .height(texture.height as u32)
        .layers(1)
        .render_pass(render_pass.render_pass);

    unsafe {
        context
            .device
            .create_framebuffer(&create_info, None)
            .expect("Unable to create frame buffer")
    }
}

// TODO: depth/render
// let render_usage = TextureUsageFlags::OVR_TEXTURE_USAGE_COLOR_ATTACHMENT;
// let render_format = render_pass.colour_format;
// let render_usage_flags = vk::ImageUsageFlags::COLOR_ATTACHMENT
//     | vk::ImageUsageFlags::TRANSIENT_ATTACHMENT
//     | vk::ImageUsageFlags::INPUT_ATTACHMENT;
// let render_image = context.create_image(width, height, render_format, render_usage_flags);
// let render_texture =
//     Texture::new(width, height, format, render_usage, &render_image, context);

// render_texture.change_usage(context, render_usage);

// TODO: FFR
// let ffr_usage = TextureUsageFlags::OVR_TEXTURE_USAGE_FRAG_DENSITY;
// let ffr_textures = texture_swap_chain
//     .ffr_images
//     .iter()
//     .zip(texture_swap_chain.ffr_image_sizes.iter())
//     .map(|(image, extent)| {
//         Texture::new(
//             extent.width as i32,
//             extent.height as i32,
//             vk::Format::R8G8_UNORM,
//             ffr_usage,
//             image,
//             context,
//         )
//     })
//     .collect::<Vec<_>>();

// TODO: Multiview
