use ash::{
    version::{EntryV1_0, InstanceV1_0},
    vk::{self, Handle},
    Entry, Instance,
};
use ovr_mobile_sys::{
    ovrJava, ovrSystemCreateInfoVulkan, vrapi_CreateSystemVulkan, vrapi_GetDeviceExtensionsVulkan,
    vrapi_GetInstanceExtensionsVulkan, VkDevice_T, VkInstance_T, VkPhysicalDevice_T,
};
use std::ffi::{CStr, CString};

use crate::{
    debug_messenger::{get_debug_messenger_create_info, setup_debug_messenger},
    device::create_logical_device,
    physical_device::get_physical_device,
    util::cstrings_to_raw,
};

pub struct VulkanRenderer {
    // pub render_pass_single_view: RenderPass,
// pub eye_command_buffers: Vec<EyeCommandBuffer>,
// pub frame_buffers: Vec<FrameBuffer>,
// pub view_matrix: Vec<ovrMatrix4f>,
// pub projection_matrix: Vec<ovrMatrix4f>,
// pub num_eyes: usize,
}

impl VulkanRenderer {
    pub unsafe fn new(java: &ovrJava) -> Self {
        println!("[VulkanRenderer] Initialising Vulkan..");
        let (instance, entry) = vulkan_init();
        println!("[VulkanRenderer] ..done");

        let required_device_extensions = get_required_extensions();

        println!("[VulkanRenderer] Getting physical device..");
        let (physical_device, queue_family_indices) =
            get_physical_device(&instance, &entry, &required_device_extensions);
        println!("[VulkanRenderer] ..done: {:?}", physical_device);

        println!("[VulkanRenderer] Creating logical device..");
        let (device, _, _) = create_logical_device(
            &instance,
            physical_device,
            &queue_family_indices,
            &required_device_extensions,
        );
        println!("[VulkanRenderer] ..done");

        let vk_instance = instance.handle().as_raw();
        let vk_physical_device = physical_device.as_raw();
        let vk_device = device.handle().as_raw();

        let mut system_info = ovrSystemCreateInfoVulkan {
            Instance: vk_instance as *mut VkInstance_T,
            PhysicalDevice: vk_physical_device as *mut VkPhysicalDevice_T,
            Device: vk_device as *mut VkDevice_T,
        };

        println!("[VulkanRenderer] Calling vrapi_CreateSystemVulkan..");
        vrapi_CreateSystemVulkan(&mut system_info);
        println!("[VulkanRenderer] ..done. VulkanRenderer initialised.");

        Self {
            // render_pass_single_view,
            // eye_command_buffers,
            // frame_buffers,
            // view_matrix,
            // projection_matrix,
            // num_eyes,
        }
    }
}

fn get_device() -> *mut ovr_mobile_sys::VkDevice_T {
    todo!()
}

unsafe fn vulkan_init() -> (Instance, Entry) {
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

    // return vec![vk::ExtDebugUtilsFn::name().as_ptr()];
}

#[cfg(not(debug_assertions))]
fn get_extension_names() -> Vec<&'static CStr> {
    return Vec::new();
}

unsafe fn get_required_extensions() -> Vec<CString> {
    let device_extension_names = CString::new("").unwrap();
    let p = device_extension_names.into_raw();
    vrapi_GetDeviceExtensionsVulkan(p, &mut 4096);
    let device_extension_names = CString::from_raw(p);

    return device_extension_names
        .to_str()
        .unwrap()
        .split(" ")
        .map(|c| CString::new(c).unwrap())
        .collect::<Vec<_>>();
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
