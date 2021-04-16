use ash::vk;
use ovr_mobile_sys::{
    ovrJava,
    ovrSystemProperty_::{
        VRAPI_SYS_PROP_SUGGESTED_EYE_TEXTURE_HEIGHT, VRAPI_SYS_PROP_SUGGESTED_EYE_TEXTURE_WIDTH,
    },
    ovrTextureSwapChain, vrapi_CreateTextureSwapChain3, vrapi_GetSystemPropertyInt,
    vrapi_GetTextureSwapChainLength,
};

pub struct ColourSwapChain {
    texture_swapchain: ovrTextureSwapChain,
    swapchain_length: i32,
    colour_textures: Vec<vk::Image>,
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
        let colour_textures = Vec::new();

        ColourSwapChain {
            texture_swapchain: *texture_swapchain,
            swapchain_length,
            colour_textures,
        }
    }
}
