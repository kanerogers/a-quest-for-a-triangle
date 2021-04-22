use ash::vk::{self, Handle};
use ovr_mobile_sys::{
    ovrTextureSwapChain, vrapi_CreateTextureSwapChain3,
    vrapi_GetTextureSwapChainBufferFoveationVulkan, vrapi_GetTextureSwapChainBufferVulkan,
    vrapi_GetTextureSwapChainLength, VkImage,
};
use std::ptr::NonNull;

// A wrapper around VrApi's texture SwapChain.
// A "texture" is VrApi terminology for a Vulkan "Image", that is to say a buffer of data that is arranged
// for a specific purpose, either to be rendered or as some other part of the rendering pipeline.
pub struct TextureSwapChain {
    pub handle: ovrTextureSwapChain,
    pub length: i32,
    pub images: Vec<vk::Image>,
    pub fragment_density_textures: Vec<vk::Image>,
    pub fragment_density_texture_sizes: Vec<vk::Extent2D>,
    pub format: vk::Format,
}

impl TextureSwapChain {
    pub unsafe fn new(width: i32, height: i32) -> TextureSwapChain {
        println!("[TextureSwapChain] Creating TextureSwapChain..");

        // Get required parameters for texture swapchain creation
        let levels = 1;
        let images_count = 3;
        let format = vk::Format::R8G8B8A8_UNORM;

        // Create texture swapchain
        println!("[TextureSwapChain] Creating texture swapchain");

        // This handle is an opaque type provided by VrApi.
        let swapchain_handle = vrapi_CreateTextureSwapChain3(
            ovr_mobile_sys::ovrTextureType::VRAPI_TEXTURE_TYPE_2D_ARRAY,
            format.as_raw() as i64,
            width,
            height,
            levels,
            images_count,
        );

        println!("[TextureSwapChain] done!");

        let swapchain_length = vrapi_GetTextureSwapChainLength(swapchain_handle);
        assert_eq!(images_count, swapchain_length);

        let mut images = Vec::with_capacity(swapchain_length as usize);

        // These images are used for Fixed Foveated Rendering (FFR)
        let mut fragment_density_images = Vec::with_capacity(swapchain_length as usize);
        let mut fragment_density_image_sizes = Vec::with_capacity(swapchain_length as usize);

        // Retrieve images from the newly created swapchain
        for i in 0..swapchain_length as usize {
            println!("[TextureSwapChain] Getting SwapChain images..");
            let image_handle = vrapi_GetTextureSwapChainBufferVulkan(swapchain_handle, i as i32);
            images.push(vk::Image::from_raw(image_handle as u64));
            println!("[TextureSwapChain] ..done!");

            let mut fragment_density_image = vk::Image::null();
            let ptr = NonNull::new(&mut fragment_density_image).unwrap().as_ptr();
            let ptr = NonNull::new(ptr).unwrap().as_ptr() as *mut VkImage;
            let mut extent = vk::Extent2D::default();

            println!("[TextureSwapChain] Getting fragment density image..");
            let result = vrapi_GetTextureSwapChainBufferFoveationVulkan(
                swapchain_handle,
                i as i32,
                ptr,
                &mut extent.height,
                &mut extent.width,
            );

            println!("[TextureSwapChain] ..done!");
            if result != 0 {
                continue;
            }

            fragment_density_images.push(fragment_density_image.to_owned());
            fragment_density_image_sizes.push(extent);
        }

        println!("[TextureSwapChain] All done! TextureSwapChain created!");

        TextureSwapChain {
            format,
            handle: *swapchain_handle,
            length: swapchain_length,
            images,
            fragment_density_textures: fragment_density_images,
            fragment_density_texture_sizes: fragment_density_image_sizes,
        }
    }
}
