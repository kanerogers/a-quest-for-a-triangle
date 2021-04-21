use crate::queue_family_indices::QueueFamilyIndices;
use ash::{version::InstanceV1_0, vk, Entry, Instance};
use std::ffi::{CStr, CString};

pub fn get_physical_device(
    instance: &Instance,
    entry: &Entry,
    required_extensions: &Vec<CString>,
) -> (vk::PhysicalDevice, QueueFamilyIndices) {
    unsafe {
        println!("[VulkanContext] Getting physical device..");
        let devices = instance.enumerate_physical_devices().unwrap();
        let mut devices = devices
            .into_iter()
            .map(|d| get_suitability(d, instance, entry, required_extensions))
            .collect::<Vec<_>>();
        devices.sort_by_key(|i| i.0);

        let (suitability, indices, physical_device) = devices.remove(0);
        assert_ne!(suitability, 0, "Failed to find a suitable device");

        println!("[VulkanContext] ..done: {:?}", physical_device);
        (physical_device, indices)
    }
}

unsafe fn get_suitability(
    device: vk::PhysicalDevice,
    instance: &Instance,
    entry: &Entry,
    required_extensions: &Vec<CString>,
) -> (i8, QueueFamilyIndices, vk::PhysicalDevice) {
    let properties = instance.get_physical_device_properties(device);
    let indices = QueueFamilyIndices::find_queue_families(instance, device, entry);
    let has_extension_support =
        check_device_extension_support(instance, device, required_extensions);
    let has_graphics_family = indices.graphics_family.is_some();

    let mut suitability = 0;
    if properties.device_type == vk::PhysicalDeviceType::DISCRETE_GPU {
        suitability -= 5;
    }

    let suitable = has_extension_support && has_graphics_family;

    if suitable {
        suitability -= 1
    }

    (suitability, indices, device)
}

fn check_device_extension_support(
    instance: &Instance,
    device: vk::PhysicalDevice,
    required_extensions: &Vec<CString>,
) -> bool {
    let supported_extensions = unsafe {
        instance
            .enumerate_device_extension_properties(device)
            .expect("Unable to get extension properties")
    }
    .iter()
    .map(|e| unsafe { CStr::from_ptr(e.extension_name.as_ptr()) })
    .collect::<Vec<_>>();

    for required_extension in required_extensions {
        if !supported_extensions.contains(&required_extension.as_c_str()) {
            println!("Required extension: {:?} was not found", required_extension);
            return false;
        }
    }

    return true;
}
