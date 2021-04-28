use crate::vulkan_renderer;
use ash::vk::{self, Handle};
use ovr_mobile_sys::{
    ovrSwapChainCreateInfo_, ovrTextureSwapChain, vrapi_CreateTextureSwapChain4,
    vrapi_GetTextureSwapChainBufferVulkan, vrapi_GetTextureSwapChainLength,
};
use std::ptr::NonNull;

// A wrapper around VrApi's texture SwapChain.
// A "texture" is VrApi terminology for a Vulkan "Image", that is to say a buffer of data that is arranged
// for a specific purpose, either to be rendered or as some other part of the rendering pipeline.
pub struct EyeTextureSwapChain {
    pub handle: NonNull<ovrTextureSwapChain>,
    pub length: i32,
    pub display_images: Vec<vk::Image>,
}

impl EyeTextureSwapChain {
    pub unsafe fn new(width: i32, height: i32) -> EyeTextureSwapChain {
        println!("[EyeTextureSwapChain] Creating TextureSwapChain..");

        // Get required parameters for texture swapchain creation
        let levels = 1;
        let images_count = 3;

        // Create texture swapchain
        println!("[EyeTextureSwapChain] Creating texture swapchain");

        // This handle is an opaque type provided by VrApi.
        let create_info = ovrSwapChainCreateInfo_ {
            Format: vulkan_renderer::COLOUR_FORMAT.as_raw() as i64,
            Width: width,
            Height: height,
            Levels: levels,
            FaceCount: 1,
            ArraySize: 1,
            BufferCount: images_count,
            CreateFlags: 0,
            UsageFlags: 0 | 0x1,
        };
        let swapchain_handle = vrapi_CreateTextureSwapChain4(&create_info);

        println!("[EyeTextureSwapChain] done!");

        let swapchain_length = vrapi_GetTextureSwapChainLength(swapchain_handle);
        assert_eq!(images_count, swapchain_length);

        // Retrieve images from the newly created swapchain
        let display_images = (0..swapchain_length)
            .map(|i| {
                println!("[EyeTextureSwapChain] Getting SwapChain images..");
                let image_handle =
                    vrapi_GetTextureSwapChainBufferVulkan(swapchain_handle, i as i32);
                println!("[EyeTextureSwapChain] ..done!");
                vk::Image::from_raw(image_handle as u64)
            })
            .collect::<Vec<_>>();

        println!("[EyeTextureSwapChain] All done! TextureSwapChain created!");

        let handle = NonNull::new(swapchain_handle).unwrap();

        EyeTextureSwapChain {
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
