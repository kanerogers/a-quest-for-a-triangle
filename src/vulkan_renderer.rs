use std::ffi::{CStr, CString};

use ash::{
    extensions::ext,
    version::{EntryV1_0, InstanceV1_0},
    vk::{self, Handle},
    Entry, Instance,
};
use ovr_mobile_sys::{
    ovrMatrix4f, ovrSystemCreateInfoVulkan, vrapi_CreateSystemVulkan, VkInstance_T,
};

use crate::{
    debug_messenger::{get_debug_messenger_create_info, setup_debug_messenger},
    eye_command_buffer::EyeCommandBuffer,
    frame_buffer::FrameBuffer,
    render_pass::RenderPass,
};

pub struct VulkanRenderer {
    pub render_pass_single_view: RenderPass,
    pub eye_command_buffers: Vec<EyeCommandBuffer>,
    pub frame_buffers: Vec<FrameBuffer>,
    pub view_matrix: Vec<ovrMatrix4f>,
    pub projection_matrix: Vec<ovrMatrix4f>,
    pub num_eyes: usize,
}

impl VulkanRenderer {
    pub unsafe fn new(
        render_pass_single_view: RenderPass,
        eye_command_buffers: Vec<EyeCommandBuffer>,
        frame_buffers: Vec<FrameBuffer>,
        view_matrix: Vec<ovrMatrix4f>,
        projection_matrix: Vec<ovrMatrix4f>,
        num_eyes: usize,
    ) -> Self {
        let instance = create_instance();
        let vk_instance = instance.handle().as_raw();
        let PhysicalDevice = get_physical_device();
        let Device = get_device();

        let mut system_info = ovrSystemCreateInfoVulkan {
            Instance: vk_instance as *mut VkInstance_T,
            PhysicalDevice,
            Device,
        };

        unsafe {
            vrapi_CreateSystemVulkan(&mut system_info);
        }

        Self {
            render_pass_single_view,
            eye_command_buffers,
            frame_buffers,
            view_matrix,
            projection_matrix,
            num_eyes,
        }
    }
}

fn get_physical_device() -> *mut ovr_mobile_sys::VkPhysicalDevice_T {
    todo!()
}

fn get_device() -> *mut ovr_mobile_sys::VkDevice_T {
    todo!()
}

unsafe fn create_instance() -> Instance {
    let app_name = CString::new("Hello Triangle").unwrap();
    let entry = Entry::new().unwrap();
    let layer_names = get_layer_names(&entry);

    let mut debug_messenger_info = get_debug_messenger_create_info();
    let extension_names = [];

    let app_info = vk::ApplicationInfo::builder()
        .application_name(&app_name)
        .api_version(vk::make_version(1, 0, 0));
    let create_info = vk::InstanceCreateInfo::builder()
        .application_info(&app_info)
        .enabled_extension_names(&extension_names)
        .enabled_layer_names(layer_names.as_slice())
        .push_next(&mut debug_messenger_info);

    let instance = entry.create_instance(&create_info, None).unwrap();

    let (debug_utils, messenger) = setup_debug_messenger(&entry, &instance, &debug_messenger_info);

    instance
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
