use ash::{version::DeviceV1_0, vk};

use crate::{vulkan_context::VulkanContext, vulkan_renderer};

// A texture is an image, or part of an image that will be rendered to the eyes.
#[derive(Debug)]
pub struct Texture {
    pub width: i32,
    pub height: i32,
    pub depth: i32,
    pub image_layout: vk::ImageLayout,
    pub image: vk::Image,
    pub memory: vk::DeviceMemory,
    pub view: vk::ImageView,
    pub sampler: vk::Sampler,
}

impl Texture {
    pub fn new(width: i32, height: i32, image: &vk::Image, context: &VulkanContext) -> Self {
        println!("[Texture] Creating texture for {:?}", image);
        // Get the appropriate image layout for this texture.
        let src_flags = vk::AccessFlags::empty();
        let dst_flags = vk::AccessFlags::SHADER_READ;
        let old_layout = vk::ImageLayout::UNDEFINED;
        let new_layout = vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL;
        let start_stage = vk::PipelineStageFlags::TOP_OF_PIPE;
        let end_stage = vk::PipelineStageFlags::ALL_GRAPHICS;
        let setup_command_buffer = context.create_setup_command_buffer();

        context.change_image_layout(
            setup_command_buffer,
            image,
            src_flags,
            dst_flags,
            old_layout,
            new_layout,
            start_stage,
            end_stage,
        );

        context.flush_setup_command_buffer(setup_command_buffer);

        // Great! Now create an image view.
        let format = vulkan_renderer::COLOUR_FORMAT;
        let aspect_mask = vk::ImageAspectFlags::COLOR;
        let view = context.create_image_view(image, format, aspect_mask);
        let sampler;

        sampler = create_sampler(context);

        let memory = vk::DeviceMemory::null();

        println!("[Texture] ..done ");

        Self {
            width,
            height,
            depth: 1,
            image_layout: new_layout,
            image: *image,
            memory,
            view,
            sampler,
        }
    }
}

fn create_sampler(context: &VulkanContext) -> vk::Sampler {
    let mipmap_mode = vk::SamplerMipmapMode::NEAREST;
    let address_mode = vk::SamplerAddressMode::CLAMP_TO_BORDER;
    let mag_filter = vk::Filter::LINEAR;

    let create_info = vk::SamplerCreateInfo::builder()
        .mag_filter(mag_filter)
        .min_filter(mag_filter)
        .mipmap_mode(mipmap_mode)
        .address_mode_u(address_mode)
        .address_mode_v(address_mode)
        .address_mode_w(address_mode)
        .mip_lod_bias(0.0)
        .anisotropy_enable(false)
        .max_anisotropy(1.0)
        .compare_enable(false)
        .compare_op(vk::CompareOp::NEVER)
        .min_lod(0.0)
        .max_lod(1.0)
        .border_color(vk::BorderColor::FLOAT_OPAQUE_BLACK)
        .unnormalized_coordinates(false);

    unsafe {
        context
            .device
            .create_sampler(&create_info, None)
            .expect("Unable to create sampler")
    }
}
