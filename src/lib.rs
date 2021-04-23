#![allow(non_snake_case)]
mod app;
mod debug_messenger;
mod depth_buffer;
mod device;
mod eye_frame_buffer;
mod physical_device;
mod queue_family_indices;
mod render_pass;
mod texture;
mod eye_texture_swap_chain;
mod util;
mod vulkan_context;
mod vulkan_renderer;
mod eye_command_buffer;

mod lib {
    use crate::app::App;

    use ovr_mobile_sys::{
        ovrGraphicsAPI_, ovrInitParms, ovrJava, ovrJava_,
        ovrStructureType_::VRAPI_STRUCTURE_TYPE_INIT_PARMS, vrapi_Initialize, VRAPI_MAJOR_VERSION,
        VRAPI_MINOR_VERSION, VRAPI_PATCH_VERSION, VRAPI_PRODUCT_VERSION,
    };

    #[cfg_attr(target_os = "android", ndk_glue::main(backtrace = "on"))]
    fn main() {
        println!("[INIT] Welcome to a Quest for Triangle!");
        let native_activity = ndk_glue::native_activity();
        let vm_ptr = native_activity.vm();

        let vm: jni::JavaVM = unsafe { jni::JavaVM::from_raw(vm_ptr) }.unwrap();
        let env = vm.attach_current_thread_permanently().unwrap();

        let java = ovrJava {
            Vm: vm.get_java_vm_pointer(),
            Env: env.get_native_interface(),
            ActivityObject: native_activity.activity(),
        };

        init_ovr(java);
        let mut app = App::new(java);

        app.run();

        println!("Destroy requested! Bye for now!");
    }

    fn init_ovr(java: ovrJava_) -> () {
        let parms: ovrInitParms = ovrInitParms {
            Type: VRAPI_STRUCTURE_TYPE_INIT_PARMS,
            ProductVersion: VRAPI_PRODUCT_VERSION as i32,
            MajorVersion: VRAPI_MAJOR_VERSION as i32,
            MinorVersion: VRAPI_MINOR_VERSION as i32,
            PatchVersion: VRAPI_PATCH_VERSION as i32,
            GraphicsAPI: ovrGraphicsAPI_::VRAPI_GRAPHICS_API_VULKAN_1,
            Java: java,
        };
        println!("[INIT] Initialising vrapi..");
        let result = unsafe { vrapi_Initialize(&parms) };
        println!("[INIT] Done. Result: {:?}", result);
    }
}
