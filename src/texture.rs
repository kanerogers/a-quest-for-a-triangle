use ash::{version::DeviceV1_0, vk};
use bitflags::bitflags;

use crate::vulkan_context::VulkanContext;

bitflags! {
    pub struct TextureUsageFlags: u32 {
        const OVR_TEXTURE_USAGE_UNDEFINED = 1 << 0;
        const OVR_TEXTURE_USAGE_GENERAL = 1 << 1;
        const OVR_TEXTURE_USAGE_TRANSFER_SRC = 1 << 2;
        const OVR_TEXTURE_USAGE_TRANSFER_DST = 1 << 3;
        const OVR_TEXTURE_USAGE_SAMPLED = 1 << 4;
        const OVR_TEXTURE_USAGE_STORAGE = 1 << 5;
        const OVR_TEXTURE_USAGE_COLOR_ATTACHMENT = 1 << 6;
        const OVR_TEXTURE_USAGE_PRESENTATION = 1 << 7;
        const OVR_TEXTURE_USAGE_FRAG_DENSITY = 1 << 8;
    }
}

#[derive(Clone, Debug, Copy, PartialEq)]
pub enum TextureWrapMode {
    _OvrTextureWrapModeRepeat,
    OvrTextureWrapModeClampToEdge,
    OvrTextureWrapModeClampToBorder,
}

#[derive(Clone, Debug, Copy, PartialEq)]
pub enum TextureFilter {
    OvrTextureFilterNearest,
    OvrTextureFilterLinear,
    _OvrTextureFilterBilinear,
}

// A texture is an image, or part of an image that will be rendered to the eyes.
pub struct Texture {
    pub width: i32,
    pub height: i32,
    pub depth: i32,
    pub layer_count: i32,
    pub mip_count: i32,
    pub sample_count: vk::SampleCountFlags,
    pub usage: TextureUsageFlags,
    pub usage_flags: TextureUsageFlags,
    pub wrap_mode: TextureWrapMode,
    pub filter: TextureFilter,
    pub max_anisotropy: f32,
    pub color_format: vk::Format,
    pub image_layout: vk::ImageLayout,
    pub image: vk::Image,
    pub memory: vk::DeviceMemory,
    pub view: vk::ImageView,
    pub sampler: vk::Sampler,
}

impl Texture {
    pub fn new(
        width: i32,
        height: i32,
        color_format: vk::Format,
        usage: TextureUsageFlags,
        image: &vk::Image,
        context: &VulkanContext,
    ) -> Self {
        println!("[Texture] Creating texture for {:?}", image);
        // Get the appropriate image layout for this texture.
        let image_layout = if usage == TextureUsageFlags::OVR_TEXTURE_USAGE_FRAG_DENSITY {
            vk::ImageLayout::FRAGMENT_DENSITY_MAP_OPTIMAL_EXT
        } else {
            vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL
        };

        // Create an image memory barrier because.. ah.. reasons.
        // dst_flags are different between different texture types
        let dst_flags = if usage == TextureUsageFlags::OVR_TEXTURE_USAGE_FRAG_DENSITY {
            vk::AccessFlags::FRAGMENT_DENSITY_MAP_READ_EXT
        } else {
            vk::AccessFlags::SHADER_READ
        };

        context.create_image_memory_barrier(image, dst_flags, image_layout);

        // Great! Now create an image view.
        let view = create_image_view(context, image, color_format);
        let sampler;
        let mip_count = 1;
        let max_anisotropy = 1.0;
        let wrap_mode = TextureWrapMode::OvrTextureWrapModeClampToBorder;
        let filter = TextureFilter::OvrTextureFilterLinear;

        // If necessary, create a sampler.
        if usage != TextureUsageFlags::OVR_TEXTURE_USAGE_COLOR_ATTACHMENT {
            sampler = create_sampler(context, wrap_mode, filter, max_anisotropy, mip_count);
        } else {
            sampler = vk::Sampler::null();
        }

        let memory = vk::DeviceMemory::null();

        println!("[Texture] ..done ");

        Self {
            width,
            height,
            depth: 1,
            layer_count: 2,
            mip_count,
            sample_count: vk::SampleCountFlags::TYPE_1,
            usage,
            usage_flags: TextureUsageFlags::OVR_TEXTURE_USAGE_COLOR_ATTACHMENT
                | TextureUsageFlags::OVR_TEXTURE_USAGE_SAMPLED
                | TextureUsageFlags::OVR_TEXTURE_USAGE_STORAGE,
            max_anisotropy,
            wrap_mode,
            filter,
            color_format,
            image_layout,
            image: *image,
            memory,
            view,
            sampler,
        }
    }

    pub fn change_usage(&self, _context: &VulkanContext, _usage: TextureUsageFlags) -> () {
        println!("[Texture] Cowardly refusing to change usage.")
    }
}

fn create_sampler(
    context: &VulkanContext,
    wrap_mode: TextureWrapMode,
    filter: TextureFilter,
    max_anisotropy: f32,
    mip_count: i32,
) -> vk::Sampler {
    let mipmap_mode = if filter == TextureFilter::OvrTextureFilterNearest {
        vk::SamplerMipmapMode::NEAREST
    } else {
        vk::SamplerMipmapMode::LINEAR
    };

    let address_mode = if wrap_mode == TextureWrapMode::OvrTextureWrapModeClampToEdge {
        vk::SamplerAddressMode::CLAMP_TO_EDGE
    } else if wrap_mode == TextureWrapMode::OvrTextureWrapModeClampToBorder {
        vk::SamplerAddressMode::CLAMP_TO_BORDER
    } else {
        vk::SamplerAddressMode::REPEAT
    };

    let mag_filter = if filter == TextureFilter::OvrTextureFilterNearest {
        vk::Filter::NEAREST
    } else {
        vk::Filter::LINEAR
    };

    let create_info = vk::SamplerCreateInfo::builder()
        .mag_filter(mag_filter)
        .min_filter(mag_filter)
        .mipmap_mode(mipmap_mode)
        .address_mode_u(address_mode)
        .address_mode_v(address_mode)
        .address_mode_w(address_mode)
        .anisotropy_enable(false)
        .max_anisotropy(max_anisotropy)
        .compare_enable(false)
        .compare_op(vk::CompareOp::NEVER)
        .min_lod(0.0)
        .max_lod(mip_count as f32)
        .border_color(vk::BorderColor::FLOAT_OPAQUE_BLACK)
        .unnormalized_coordinates(false);

    unsafe {
        context
            .device
            .create_sampler(&create_info, None)
            .expect("Unable to create sampler")
    }
}

fn create_image_view(
    context: &VulkanContext,
    image: &vk::Image,
    color_format: vk::Format,
) -> vk::ImageView {
    let components = vk::ComponentMapping::builder()
        .r(vk::ComponentSwizzle::R)
        .g(vk::ComponentSwizzle::G)
        .b(vk::ComponentSwizzle::B)
        .a(vk::ComponentSwizzle::A)
        .build();

    let subresource_range = vk::ImageSubresourceRange::builder()
        .aspect_mask(vk::ImageAspectFlags::COLOR)
        .level_count(1)
        .layer_count(2)
        .build();

    let create_info = vk::ImageViewCreateInfo::builder()
        .image(*image)
        .view_type(vk::ImageViewType::TYPE_2D_ARRAY)
        .format(color_format)
        .components(components)
        .subresource_range(subresource_range);

    unsafe {
        context
            .device
            .create_image_view(&create_info, None)
            .expect("Unable to create image view")
    }
}
