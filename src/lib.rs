use log::debug;
use ovr_mobile_sys::ovrJava;
use ovr_mobile_sys::ovrStructureType_::VRAPI_STRUCTURE_TYPE_INIT_PARMS;
use ovr_mobile_sys::VRAPI_PRODUCT_VERSION;
use ovr_mobile_sys::{ovrGraphicsAPI_, ovrInitParms};
use ovr_mobile_sys::{VRAPI_MAJOR_VERSION, VRAPI_MINOR_VERSION, VRAPI_PATCH_VERSION};

#[cfg(target_os = "android")]
fn get_java() -> ovrJava {
    let native_activity = ndk_glue::native_activity();
    let vm_ptr = native_activity.vm();
    let vm = unsafe { jni::JavaVM::from_raw(vm_ptr) };
    return vm;
    return ovrJava {
        Vm: vm,
        Env: vm.attatch_current_thread(),
        ActivityObject: native_activity.activity(),
    };
}

#[cfg(not(target_os = "android"))]
fn get_java() -> ovrJava {
    unimplemented!();
}

#[cfg_attr(target_os = "android", ndk_glue::main(backtrace = "on"))]
fn main() {
    env_logger::init();
    println!("bonjour tout le monde");
    debug!("hello from debug logger");

    let java = get_java();
    let params = ovrInitParms {
        GraphicsAPI: ovrGraphicsAPI_::VRAPI_GRAPHICS_API_OPENGL_ES_2,
        MajorVersion: VRAPI_MAJOR_VERSION as i32,
        MinorVersion: VRAPI_MINOR_VERSION as i32,
        PatchVersion: VRAPI_PATCH_VERSION as i32,
        ProductVersion: VRAPI_PRODUCT_VERSION as i32,
        Type: VRAPI_STRUCTURE_TYPE_INIT_PARMS,
        Java: java,
    };

    debug!("Hello!");
}
