use ash::{extensions::ext, vk, Entry, Instance};
use std::ffi::CStr;

#[cfg(debug_assertions)]
pub fn setup_debug_messenger(
    entry: &Entry,
    instance: &Instance,
    info: &vk::DebugUtilsMessengerCreateInfoEXT,
) -> (Option<ext::DebugUtils>, Option<vk::DebugUtilsMessengerEXT>) {
    let debug_utils = ext::DebugUtils::new(entry, instance);

    let messenger = unsafe {
        debug_utils
            .create_debug_utils_messenger(info, None)
            .unwrap()
    };

    println!("[DebugMessenger] Created messenger: {:?}", messenger);

    (Some(debug_utils), Some(messenger))
}

pub fn get_debug_messenger_create_info() -> vk::DebugUtilsMessengerCreateInfoEXTBuilder<'static> {
    let message_severity = vk::DebugUtilsMessageSeverityFlagsEXT::all();
    let message_type = vk::DebugUtilsMessageTypeFlagsEXT::all();
    vk::DebugUtilsMessengerCreateInfoEXT::builder()
        .message_severity(message_severity)
        .message_type(message_type)
        .pfn_user_callback(Some(debug_messenger_callback))
}

#[cfg(debug_assertions)]
unsafe extern "system" fn debug_messenger_callback(
    _message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    _message_types: vk::DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _p_user_data: *mut std::ffi::c_void,
) -> vk::Bool32 {
    let data = *p_callback_data;
    let message_id = data.message_id_number;

    // These messages are suspected to be bugs. TODO: investigate
    if message_id == 0x1510053d
        || message_id == 0x4d5b752
        || message_id == 0x255d7463
        || message_id == 0x2f90f26b
        || message_id == 0x156f5810
    {
        return vk::FALSE;
    }

    println!("[DebugMessenger]: {:?}", CStr::from_ptr(data.p_message));

    return vk::FALSE;
}

#[cfg(not(debug_assertions))]
fn setup_debug_messenger(
    entry: &Entry,
    instance: &Instance,
) -> (Option<ext::DebugUtils>, Option<vk::DebugUtilsMessengerEXT>) {
    (None, None)
}
