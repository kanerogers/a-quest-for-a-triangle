use std::cmp::max;

use ash::{
    version::{DeviceV1_0, InstanceV1_0},
    vk,
};
use ovr_mobile_sys::ovrTextureSwapChain;

use crate::{render_pass::RenderPass, vulkan_context::VulkanContext};
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
        let render_image = create_render_image(width, height, context, render_pass);
        let render_texture =
            Texture::new(width, height, format, render_usage, &render_image, context);
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
    println!("[FrameBuffer] Creating render image");
    let face_count = 1;
    let max_dimension = max(width, height);
    let format = render_pass.colour_format;
    let format_properties = unsafe {
        context
            .instance
            .get_physical_device_format_properties(context.physical_device, format)
    };

    assert!(format_properties
        .optimal_tiling_features
        .contains(vk::FormatFeatureFlags::COLOR_ATTACHMENT));

    let num_storage_levels = 1;
    let array_layers_count = face_count;
    let usage = vk::ImageUsageFlags::COLOR_ATTACHMENT
        | vk::ImageUsageFlags::TRANSIENT_ATTACHMENT
        | vk::ImageUsageFlags::INPUT_ATTACHMENT;
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

    let memory_requirements = unsafe { context.device.get_image_memory_requirements(image) };
    let memory_type = memory_requirements.memory_type_bits;
    let memory_flags = vk::MemoryPropertyFlags::DEVICE_LOCAL;
    let memory_type_index = get_memory_type_index(context, memory_type, memory_flags);
    let allocation_size = memory_requirements.size;
    let memory_allocate_info = vk::MemoryAllocateInfo::builder()
        .allocation_size(allocation_size)
        .memory_type_index(memory_type_index);

    println!("[FrameBuffer] Creating device memory..");
    let device_memory = unsafe {
        context
            .device
            .allocate_memory(&memory_allocate_info, None)
            .expect("Unable to allocate memory")
    };
    println!("[FrameBuffer] ..done. Binding memory..");
    unsafe {
        context
            .device
            .bind_image_memory(image, device_memory, 0)
            .expect("Unable to bind image memory")
    };

    println!("[FrameBuffer] ..done. created render image: {:?}", image);
    image
}

fn get_memory_type_index(
    context: &VulkanContext,
    required_memory_type_bits: u32,
    required_memory_flags: vk::MemoryPropertyFlags,
) -> u32 {
    let properties = unsafe {
        context
            .instance
            .get_physical_device_memory_properties(context.physical_device)
    };
    for memory_index in 0..properties.memory_type_count {
        let memory_type_bits = 1 << memory_index;
        let is_required_memory_type = required_memory_type_bits & memory_type_bits == 1;
        let memory_flags = properties.memory_types[memory_index as usize].property_flags;
        let has_required_properties = memory_flags.contains(required_memory_flags);

        if is_required_memory_type && has_required_properties {
            return memory_index;
        }
    }
    panic!("Unable to find suitable memory type index");
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
