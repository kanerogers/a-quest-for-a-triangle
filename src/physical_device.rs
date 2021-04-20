use crate::queue_family_indices::QueueFamilyIndices;
use ash::{version::InstanceV1_0, vk, Entry, Instance};
use ovr_mobile_sys::{ovrJava, vrapi_GetDeviceExtensionsVulkan};
use std::ffi::{CStr, CString};

pub fn get_physical_device(
    instance: &Instance,
    entry: &Entry,
    java: &ovrJava,
) -> (vk::PhysicalDevice, QueueFamilyIndices) {
    unsafe {
        let devices = instance.enumerate_physical_devices().unwrap();
        let mut devices = devices
            .into_iter()
            .map(|d| get_suitability(d, instance, entry, java))
            .collect::<Vec<_>>();
        devices.sort_by_key(|i| i.0);

        let (_, indices, device) = devices.remove(0);
        (device, indices)
    }
}

unsafe fn get_suitability(
    device: vk::PhysicalDevice,
    instance: &Instance,
    entry: &Entry,
    java: &ovrJava,
) -> (i8, QueueFamilyIndices, vk::PhysicalDevice) {
    let properties = instance.get_physical_device_properties(device);
    let indices = QueueFamilyIndices::find_queue_families(instance, device, entry);
    let required_extensions = get_required_extensions(java);
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

unsafe fn get_required_extensions(java: &ovr_mobile_sys::ovrJava_) -> Vec<CString> {
    let device_extension_names = CString::new("").unwrap();
    let p = device_extension_names.into_raw();
    vrapi_GetDeviceExtensionsVulkan(p, &mut 4096);
    let device_extension_names = CString::from_raw(p);

    device_extension_names
        .to_str()
        .unwrap()
        .split(" ")
        .map(|c| CString::new(c).unwrap())
        .collect()
}

fn check_device_extension_support(
    instance: &Instance,
    device: vk::PhysicalDevice,
    required_extensions: Vec<CString>,
) -> bool {
    let extensions = unsafe {
        instance
            .enumerate_device_extension_properties(device)
            .expect("Unable to get extension properties")
    }
    .iter()
    .map(|e| unsafe { CStr::from_ptr(e.extension_name.as_ptr()) })
    .collect::<Vec<_>>();

    let mut has_extension = false;

    for required_extension in required_extensions {
        has_extension = extensions.contains(&required_extension.as_c_str());
    }

    return has_extension;
}
