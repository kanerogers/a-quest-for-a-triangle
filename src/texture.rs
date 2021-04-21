use ash::vk;
use bitflags::bitflags;

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

pub enum TextureWrapMode {
    OvrTextureWrapModeRepeat,
    OvrTextureWrapModeClampToEdge,
    OvrTextureWrapModeClampToBorder,
}

pub enum TextureFilter {
    OvrTextureFilterNearest,
    OvrTextureFilterLinear,
    OvrTextureFilterBilinear,
}

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
    pub max_aniostropy: f32,
    pub color_format: vk::Format,
    pub image_layout: vk::ImageLayout,
    pub image: vk::Image,
    pub memory: vk::DeviceMemory,
    pub view: vk::ImageView,
    pub sampler: vk::Sampler,
}

impl Texture {
    pub fn new(width: i32, height: i32, color_format: vk::Format, image: vk::Image) -> Self {
        let memory = vk::DeviceMemory::null();
        let sampler = vk::Sampler::null();
        let view = vk::ImageView::null();

        Self {
            width,
            height,
            depth: 1,
            layer_count: 2,
            mip_count: 1,
            sample_count: vk::SampleCountFlags::TYPE_1,
            usage: TextureUsageFlags::OVR_TEXTURE_USAGE_SAMPLED,
            usage_flags: TextureUsageFlags::OVR_TEXTURE_USAGE_COLOR_ATTACHMENT
                | TextureUsageFlags::OVR_TEXTURE_USAGE_SAMPLED
                | TextureUsageFlags::OVR_TEXTURE_USAGE_STORAGE,
            wrap_mode: TextureWrapMode::OvrTextureWrapModeClampToBorder,
            filter: TextureFilter::OvrTextureFilterLinear,
            max_aniostropy: 1.0,
            color_format,
            image_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
            image,
            memory,
            view,
            sampler,
        }
    }
}
