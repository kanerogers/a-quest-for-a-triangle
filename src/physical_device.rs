use crate::queue_family_indices::QueueFamilyIndices;
use ash::{version::InstanceV1_0, vk, Entry, Instance};
use std::ffi::CStr;

pub fn get_physical_device(instance: &Instance, entry: &Entry) -> vk::PhysicalDevice {
    unsafe {
        let devices = instance.enumerate_physical_devices().unwrap();
        let mut devices = devices
            .into_iter()
            .map(|d| get_suitability(d, instance, entry))
            .collect::<Vec<_>>();
        devices.sort_by_key(|i| i.0);

        let (_, indices, device) = devices.remove(0);
        device
    }
}

unsafe fn get_suitability(
    device: vk::PhysicalDevice,
    instance: &Instance,
    entry: &Entry,
) -> (i8, QueueFamilyIndices, vk::PhysicalDevice) {
    let properties = instance.get_physical_device_properties(device);
    let indices = QueueFamilyIndices::find_queue_families(instance, device, entry);
    let has_extension_support = check_device_extension_support(instance, device);
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

fn check_device_extension_support(instance: &Instance, device: vk::PhysicalDevice) -> bool {
    let extensions = unsafe {
        instance
            .enumerate_device_extension_properties(device)
            .expect("Unable to get extension properties")
    }
    .iter()
    .map(|e| unsafe { CStr::from_ptr(e.extension_name.as_ptr()) })
    .collect::<Vec<_>>();

    let required_extension = todo!();

    extensions.contains(&required_extension)
}
