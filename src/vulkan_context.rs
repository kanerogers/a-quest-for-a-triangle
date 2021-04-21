use crate::{
    debug_messenger::{get_debug_messenger_create_info, setup_debug_messenger},
    device::create_logical_device,
    physical_device::get_physical_device,
    util::cstrings_to_raw,
};
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

#[derive(Clone)]
pub struct VulkanContext {
    pub entry: Entry,
    pub instance: Instance,
    pub device: Device,
    pub physical_device: vk::PhysicalDevice,
    pub graphics_queue: vk::Queue,
    pub present_queue: vk::Queue,
}

impl std::fmt::Debug for VulkanContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VulkanContext")
            .field("physical_device", &self.physical_device)
            .finish()
    }
}

impl VulkanContext {
    pub unsafe fn new() -> Self {
        let (instance, entry) = vulkan_init();
        let required_device_extensions = get_required_device_extensions();

        let (physical_device, queue_family_indices) =
            get_physical_device(&instance, &required_device_extensions);

        let (device, graphics_queue, present_queue) = create_logical_device(
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
            graphics_queue,
            present_queue,
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
    println!("[VulkanContext] Calling vrapi_CreateSystemVulkan..");
    unsafe { vrapi_CreateSystemVulkan(&mut system_info) };
    println!("[VulkanContext] ..done. VulkanRenderer initialised.");
}

unsafe fn vulkan_init() -> (Instance, Entry) {
    println!("[VulkanContext] Initialising Vulkan..");
    let app_name = CString::new("A Quest for a Triangle").unwrap();
    let entry = Entry::new().unwrap();
    let layer_names = get_layer_names(&entry);
    let layer_names_raw = cstrings_to_raw(&layer_names);

    let mut debug_messenger_info = get_debug_messenger_create_info();
    let extension_names = get_instance_extensions();
    let extension_names_raw = cstrings_to_raw(&extension_names);

    let app_info = vk::ApplicationInfo::builder()
        .application_name(&app_name)
        .api_version(vk::make_version(1, 0, 0));
    let create_info = vk::InstanceCreateInfo::builder()
        .application_info(&app_info)
        .enabled_extension_names(&extension_names_raw)
        .enabled_layer_names(&layer_names_raw)
        .push_next(&mut debug_messenger_info);

    let instance = entry.create_instance(&create_info, None).unwrap();

    let (_debug_utils, _messenger) =
        setup_debug_messenger(&entry, &instance, &debug_messenger_info);
    println!("[VulkanContext] ..done");

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

    return extensions;
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

fn get_layer_names(entry: &Entry) -> Vec<CString> {
    let validation_layers = get_validation_layers();
    let supported_layers = entry
        .enumerate_instance_layer_properties()
        .expect("Unable to enumerate instance layer properties")
        .iter()
        .map(|l| unsafe { CStr::from_ptr(l.layer_name.as_ptr()) })
        .collect::<Vec<_>>();

    for layer in &validation_layers {
        assert!(supported_layers.contains(&layer.as_c_str()));
    }

    return validation_layers;
}

#[cfg(debug_assertions)]
fn get_validation_layers() -> Vec<CString> {
    let validation_layer = CString::new("VK_LAYER_KHRONOS_validation").unwrap();
    return vec![validation_layer];
}

#[cfg(not(debug_assertions))]
fn get_validation_layers() -> Vec<CString> {
    return Vec::new();
}
