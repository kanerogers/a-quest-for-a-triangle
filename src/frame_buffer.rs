use std::cmp::max;

use ash::{
    version::{DeviceV1_0, InstanceV1_0},
    vk,
};
use ovr_mobile_sys::ovrTextureSwapChain;

use crate::{render_pass::RenderPass, util::log_2, vulkan_context::VulkanContext};
use crate::{
    texture::{Texture, TextureUsageFlags},
    texture_swap_chain::TextureSwapChain,
};

pub struct FrameBuffer {
    pub width: i32,
    pub height: i32,
    pub swapchain_handle: ovrTextureSwapChain,
    pub swap_chain_length: i32,
    pub display_textures: Vec<Texture>, // textures that will be displayed to the user's eyes
    // pub render_texture: Texture,        // ??
    pub framebuffers: Vec<vk::Framebuffer>, // ??
    pub num_layers: usize,
    pub current_buffer: usize,
    pub current_layer: usize,
}

impl FrameBuffer {
    pub fn new(
        texture_swap_chain: &TextureSwapChain,
        render_pass: &RenderPass,
        context: &VulkanContext,
        width: i32,
        height: i32,
    ) -> Self {
        println!("[FrameBuffer] Creating FrameBuffer..");
        let texture_swap_chain_length = texture_swap_chain.length;
        let format = texture_swap_chain.format;
        let display_usage = TextureUsageFlags::OVR_TEXTURE_USAGE_SAMPLED;
        let display_textures = texture_swap_chain
            .display_images
            .iter()
            .map(|image| Texture::new(width, height, format, display_usage, image, context))
            .collect::<Vec<_>>();

        let render_usage = TextureUsageFlags::OVR_TEXTURE_USAGE_COLOR_ATTACHMENT;
        // let render_image = create_render_image(width, height, context, render_pass);
        // let render_texture =
        //     Texture::new(width, height, format, render_usage, &render_image, context);
        // render_texture.change_usage(context, render_usage);

        let mut framebuffers = Vec::new();

        println!("[FrameBuffer] Done!");

        Self {
            width,
            height,
            swapchain_handle: texture_swap_chain.handle,
            swap_chain_length: texture_swap_chain_length,
            display_textures,
            // render_texture,
            framebuffers,
            num_layers: 2,
            current_buffer: 0,
            current_layer: 0,
        }
    }
}

fn create_render_image(
    width: i32,
    height: i32,
    context: &VulkanContext,
    render_pass: &RenderPass,
) -> vk::Image {
    // (ovrVkTextureFormat)renderPass->internalColorFormat
    // renderPass->sampleCount
    // file_name = "data'"
    // frameBuffer->Width,
    // frameBuffer->Height,
    // 1, : mip count
    // isMultiview ? 2 : 1, : num layers
    // OVR_TEXTURE_USAGE_COLOR_ATTACHMENT);
    // depth = 0
    let face_count = 1;
    let max_dimension = max(width, height);
    let max_mip_levels = 1 + log_2(max_dimension);
    let format = render_pass.colour_format;
    let format_properties = unsafe {
        context
            .instance
            .get_physical_device_format_properties(context.physical_device, format)
    };

    assert!(format_properties
        .optimal_tiling_features
        .contains(vk::FormatFeatureFlags::COLOR_ATTACHMENT));

    let num_storage_levels = max_mip_levels;
    let array_layers_count = face_count;
    let usage = vk::ImageUsageFlags::COLOR_ATTACHMENT;
    let sample_count = vk::SampleCountFlags::TYPE_4;
    let extent = vk::Extent3D::builder()
        .width(width as u32)
        .height(height as u32)
        .depth(1)
        .build();

    let create_info = vk::ImageCreateInfo::builder()
        .image_type(vk::ImageType::TYPE_2D)
        .format(format)
        .extent(extent)
        .mip_levels(num_storage_levels)
        .array_layers(array_layers_count)
        .samples(sample_count)
        .tiling(vk::ImageTiling::OPTIMAL)
        .usage(usage)
        .sharing_mode(vk::SharingMode::EXCLUSIVE)
        .initial_layout(vk::ImageLayout::UNDEFINED);

    let image = unsafe {
        context
            .device
            .create_image(&create_info, None)
            .expect("Unable to create render image")
    };

    image
}

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
