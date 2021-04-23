use std::ptr::NonNull;

use ash::vk::{self, Handle};
use ovr_mobile_sys::{
    ovrTextureSwapChain, vrapi_CreateTextureSwapChain3, vrapi_GetTextureSwapChainBufferVulkan,
    vrapi_GetTextureSwapChainLength,
};

// A wrapper around VrApi's texture SwapChain.
// A "texture" is VrApi terminology for a Vulkan "Image", that is to say a buffer of data that is arranged
// for a specific purpose, either to be rendered or as some other part of the rendering pipeline.
pub struct EyeTextureSwapChain {
    pub handle: NonNull<ovrTextureSwapChain>,
    pub length: i32,
    pub display_images: Vec<vk::Image>,
    pub format: vk::Format,
}

impl EyeTextureSwapChain {
    pub unsafe fn new(width: i32, height: i32) -> EyeTextureSwapChain {
        println!("[EyeTextureSwapChain] Creating TextureSwapChain..");

        // Get required parameters for texture swapchain creation
        let levels = 1;
        let images_count = 3;
        let format = vk::Format::R8G8B8A8_UNORM;

        // Create texture swapchain
        println!("[EyeTextureSwapChain] Creating texture swapchain");

        // This handle is an opaque type provided by VrApi.
        let swapchain_handle = vrapi_CreateTextureSwapChain3(
            ovr_mobile_sys::ovrTextureType::VRAPI_TEXTURE_TYPE_2D,
            format.as_raw() as i64,
            width,
            height,
            levels,
            images_count,
        );

        println!("[EyeTextureSwapChain] done!");

        let swapchain_length = vrapi_GetTextureSwapChainLength(swapchain_handle);
        assert_eq!(images_count, swapchain_length);

        // These are the images that will be ultimately displayed to the user's eyes.
        let mut display_images = Vec::with_capacity(swapchain_length as usize);

        // Retrieve images from the newly created swapchain
        for i in 0..swapchain_length as usize {
            println!("[EyeTextureSwapChain] Getting SwapChain images..");
            let image_handle = vrapi_GetTextureSwapChainBufferVulkan(swapchain_handle, i as i32);
            display_images.push(vk::Image::from_raw(image_handle as u64));
            println!("[EyeTextureSwapChain] ..done!");
        }

        println!("[EyeTextureSwapChain] All done! TextureSwapChain created!");

        let handle = NonNull::new(swapchain_handle).unwrap();

        EyeTextureSwapChain {
            format,
            handle,
            length: swapchain_length,
            display_images,
        }
    }
}

// TODO: FFR
// These images are used for Fixed Foveated Rendering (FFR)
// let mut ffr_images = Vec::with_capacity(swapchain_length as usize);
// let mut ffr_image_sizes = Vec::with_capacity(swapchain_length as usize);
// let mut ffr_image = vk::Image::null();
// let ptr = NonNull::new(&mut ffr_image).unwrap().as_ptr();
// let ptr = NonNull::new(ptr).unwrap().as_ptr() as *mut VkImage;
// let mut ffr_image_size = vk::Extent2D::default();

// println!("[EyeTextureSwapChain] Getting fragment density image..");
// let result = vrapi_GetTextureSwapChainBufferFoveationVulkan(
//     swapchain_handle,
//     i as i32,
//     ptr,
//     &mut ffr_image_size.height,
//     &mut ffr_image_size.width,
// );

// println!("[EyeTextureSwapChain] ..done!");
// if result != 0 {
//     continue;
// }
