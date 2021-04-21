use ash::{
    version::{EntryV1_0, InstanceV1_0},
    vk::{self, Handle},
    Device, Entry, Instance,
};
use ovr_mobile_sys::{
    ovrSystemCreateInfoVulkan, vrapi_CreateSystemVulkan, vrapi_GetDeviceExtensionsVulkan,
    vrapi_GetInstanceExtensionsVulkan, VkDevice_T, VkInstance_T, VkPhysicalDevice_T,
};
use std::ffi::{CStr, CString};

use crate::{
    debug_messenger::{get_debug_messenger_create_info, setup_debug_messenger},
    device::create_logical_device,
    physical_device::get_physical_device,
    util::cstrings_to_raw,
};

pub struct Context {
    pub entry: Entry,
    pub instance: Instance,
    pub device: Device,
    pub physical_device: vk::PhysicalDevice,
}

impl Context {
    pub unsafe fn new() -> Self {
        let (instance, entry) = vulkan_init();
        let required_device_extensions = get_required_device_extensions();

        let (physical_device, queue_family_indices) =
            get_physical_device(&instance, &entry, &required_device_extensions);

        let (device, _, _) = create_logical_device(
            &instance,
            physical_device,
            &queue_family_indices,
            &required_device_extensions,
        );

        create_system_vulkan(&instance, physical_device, &device);

        Self {
            entry,
            instance,
            device,
            physical_device,
        }
    }
}

fn create_system_vulkan(instance: &Instance, physical_device: vk::PhysicalDevice, device: &Device) {
    let vk_instance = instance.handle().as_raw();
    let vk_physical_device = physical_device.as_raw();
    let vk_device = device.handle().as_raw();
    let mut system_info = ovrSystemCreateInfoVulkan {
        Instance: vk_instance as *mut VkInstance_T,
        PhysicalDevice: vk_physical_device as *mut VkPhysicalDevice_T,
        Device: vk_device as *mut VkDevice_T,
    };
    println!("[VulkanRenderer] Calling vrapi_CreateSystemVulkan..");
    unsafe { vrapi_CreateSystemVulkan(&mut system_info) };
    println!("[VulkanRenderer] ..done. VulkanRenderer initialised.");
}

unsafe fn vulkan_init() -> (Instance, Entry) {
    println!("[VulkanRenderer] Initialising Vulkan..");
    let app_name = CString::new("A Quest for a Triangle").unwrap();
    let entry = Entry::new().unwrap();
    let layer_names = get_layer_names(&entry);

    let mut debug_messenger_info = get_debug_messenger_create_info();
    let extension_names = get_instance_extensions();
    let extension_names_raw = cstrings_to_raw(&extension_names);

    let app_info = vk::ApplicationInfo::builder()
        .application_name(&app_name)
        .api_version(vk::make_version(1, 0, 0));
    let create_info = vk::InstanceCreateInfo::builder()
        .application_info(&app_info)
        .enabled_extension_names(&extension_names_raw)
        .enabled_layer_names(layer_names.as_slice())
        .push_next(&mut debug_messenger_info);

    let instance = entry.create_instance(&create_info, None).unwrap();

    let (debug_utils, messenger) = setup_debug_messenger(&entry, &instance, &debug_messenger_info);
    println!("[VulkanRenderer] ..done");

    (instance, entry)
}

#[cfg(debug_assertions)]
fn get_instance_extensions() -> Vec<CString> {
    let instance_extension_names = CString::new("").unwrap();
    let p = instance_extension_names.into_raw();
    unsafe { vrapi_GetInstanceExtensionsVulkan(p, &mut 4096) };
    let instance_extension_names = unsafe { CString::from_raw(p) };

    let mut extensions = instance_extension_names
        .to_str()
        .unwrap()
        .split(" ")
        .map(|c| CString::new(c).unwrap())
        .collect::<Vec<_>>();

    extensions.push(vk::KhrGetPhysicalDeviceProperties2Fn::name().to_owned());
    extensions.push(vk::ExtDebugUtilsFn::name().to_owned());

    for ext in &extensions {
        println!("ext: {:?}", ext);
    }

    return extensions;

    // return vec![vk::ExtDebugUtilsFn::name().as_ptr()];
}

#[cfg(not(debug_assertions))]
fn get_extension_names() -> Vec<&'static CStr> {
    return Vec::new();
}

unsafe fn get_required_device_extensions() -> Vec<CString> {
    let device_extension_names = CString::new("").unwrap();
    let p = device_extension_names.into_raw();
    vrapi_GetDeviceExtensionsVulkan(p, &mut 4096);
    let device_extension_names = CString::from_raw(p);

    let names = device_extension_names
        .to_str()
        .unwrap()
        .split(" ")
        .map(|c| CString::new(c).unwrap())
        .collect::<Vec<_>>();

    return names;
}

fn get_layer_names(entry: &Entry) -> Vec<*const u8> {
    let mut validation_layers_raw = Vec::new();
    if !should_add_validation_layers() {
        return validation_layers_raw;
    };

    let validation_layers = get_validation_layers();
    let supported_layers = entry
        .enumerate_instance_layer_properties()
        .expect("Unable to enumerate instance layer properties")
        .iter()
        .map(|l| unsafe { CStr::from_ptr(l.layer_name.as_ptr()) })
        .collect::<Vec<_>>();

    for layer in validation_layers {
        assert!(
            supported_layers.contains(&layer),
            "Unsupported layer: {:?}",
            layer
        );
        validation_layers_raw.push(layer.as_ptr() as *const u8)
    }

    return validation_layers_raw;
}

#[cfg(debug_assertions)]
fn get_validation_layers() -> Vec<&'static CStr> {
    let validation_layer = CStr::from_bytes_with_nul(b"VK_LAYER_KHRONOS_validation\0").unwrap();
    return vec![validation_layer];
}

#[cfg(not(debug_assertions))]
fn get_validation_layers() -> Vec<&'static CStr> {
    return Vec::new();
}

#[cfg(debug_assertions)]
fn should_add_validation_layers() -> bool {
    true
}

#[cfg(not(debug_assertions))]
fn should_add_validation_layers() -> bool {
    false
}
