use ash::vk::{self, Handle};
use ovr_mobile_sys::{
    ovrJava,
    ovrSuccessResult_::ovrSuccess,
    ovrSystemProperty_::{
        VRAPI_SYS_PROP_SUGGESTED_EYE_TEXTURE_HEIGHT, VRAPI_SYS_PROP_SUGGESTED_EYE_TEXTURE_WIDTH,
    },
    ovrTextureSwapChain, vrapi_CreateTextureSwapChain3, vrapi_GetSystemPropertyInt,
    vrapi_GetTextureSwapChainBufferFoveationVulkan, vrapi_GetTextureSwapChainBufferVulkan,
    vrapi_GetTextureSwapChainLength, VkImage,
};

pub struct ColourSwapChain {
    texture_swapchain: ovrTextureSwapChain,
    swapchain_length: i32,
    colour_textures: Vec<vk::Image>,
    fragment_density_textures: Vec<vk::Image>,
    fragment_density_texture_sizes: Vec<vk::Extent2D>,
}

impl ColourSwapChain {
    pub unsafe fn init(java: &ovrJava) -> ColourSwapChain {
        let width = vrapi_GetSystemPropertyInt(java, VRAPI_SYS_PROP_SUGGESTED_EYE_TEXTURE_WIDTH);
        let height = vrapi_GetSystemPropertyInt(java, VRAPI_SYS_PROP_SUGGESTED_EYE_TEXTURE_HEIGHT);
        let levels = 1;
        let buffer_count = 3;
        let texture_swapchain = vrapi_CreateTextureSwapChain3(
            ovr_mobile_sys::ovrTextureType::VRAPI_TEXTURE_TYPE_2D_ARRAY,
            vk::Format::R8G8B8A8_UNORM.as_raw() as i64,
            width,
            height,
            levels,
            buffer_count,
        );

        let swapchain_length = vrapi_GetTextureSwapChainLength(texture_swapchain);
        let mut colour_textures = Vec::with_capacity(swapchain_length as usize);
        let mut fragment_density_textures = Vec::with_capacity(swapchain_length as usize);
        let mut fragment_density_texture_sizes = Vec::with_capacity(swapchain_length as usize);

        for i in 0..swapchain_length as usize {
            let colour_texture = vrapi_GetTextureSwapChainBufferVulkan(texture_swapchain, i as i32);
            colour_textures.insert(i, vk::Image::from_raw(colour_texture as u64));

            let mut image = vk::Image::null();
            let image: *mut vk::Image = &mut image;
            let mut extent = vk::Extent2D::default();
            let result = vrapi_GetTextureSwapChainBufferFoveationVulkan(
                texture_swapchain,
                i as i32,
                image as *mut VkImage,
                &mut extent.height,
                &mut extent.width,
            );

            if result != ovrSuccess as i32 {
                continue;
            };

            fragment_density_textures.insert(i, *image);
            fragment_density_texture_sizes.insert(i, extent);
        }

        ColourSwapChain {
            texture_swapchain: *texture_swapchain,
            swapchain_length,
            colour_textures,
            fragment_density_textures,
            fragment_density_texture_sizes,
        }
    }
}
