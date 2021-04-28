use crate::util::cstrings_to_raw;
use std::ffi::CString;

use ash::{
    version::{DeviceV1_0, InstanceV1_0},
    vk, Device, Instance,
};

use crate::queue_family_indices::QueueFamilyIndices;

// Logical Device
pub fn create_logical_device(
    instance: &Instance,
    physical_device: vk::PhysicalDevice,
    indices: &QueueFamilyIndices,
    required_extensions: &Vec<CString>,
) -> (Device, vk::Queue, vk::Queue) {
    println!("[VulkanContext] Creating logical device.. ");

    let queue_priorities = [0.5];
    let required_extensions_raw = cstrings_to_raw(required_extensions);
    let graphics_queue_create_info = vk::DeviceQueueCreateInfo::builder()
        .queue_priorities(&queue_priorities)
        .queue_family_index(indices.graphics_family.unwrap())
        .build();

    let queue_create_infos = [graphics_queue_create_info];

    let physical_device_features = vk::PhysicalDeviceFeatures::builder();
    let device_create_info = vk::DeviceCreateInfo::builder()
        .queue_create_infos(&queue_create_infos)
        .enabled_extension_names(&required_extensions_raw)
        .enabled_features(&physical_device_features);

    let device =
        unsafe { instance.create_device(physical_device, &device_create_info, None) }.unwrap();

    let graphics_queue = unsafe { device.get_device_queue(indices.graphics_family.unwrap(), 0) };
    let present_queue = unsafe { device.get_device_queue(indices.present_family.unwrap(), 0) };

    println!("[VulkanContext] ..done");

    (device, graphics_queue, present_queue)
}
