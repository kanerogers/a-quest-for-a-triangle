#[cfg(target_os = "android")]
mod lib {
    use ndk::looper::{Poll, ThreadLooper};
    use ovr_mobile_sys::ovrStructureType_::VRAPI_STRUCTURE_TYPE_INIT_PARMS;
    use ovr_mobile_sys::vrapi_Initialize;
    use ovr_mobile_sys::VRAPI_PRODUCT_VERSION;
    use ovr_mobile_sys::{ovrGraphicsAPI_, ovrInitParms};
    use ovr_mobile_sys::{ovrJava, ovrMobile};
    use ovr_mobile_sys::{VRAPI_MAJOR_VERSION, VRAPI_MINOR_VERSION, VRAPI_PATCH_VERSION};
    use std::time::Duration;

    pub const LOOPER_ID_MAIN: u32 = 0;
    pub const LOOPER_ID_INPUT: u32 = 0;
    pub const LOOPER_TIMEOUT: Duration = Duration::from_millis(0u64);

    struct App {
        java: ovrJava,
        ovrMobile: Option<ovrMobile>,
        destroyRequested: bool,
        resumed: bool,
    }

    pub fn poll_all_ms(block: bool) -> Option<ndk_glue::Event> {
        let looper = ThreadLooper::for_thread().unwrap();
        let result = if block {
            looper.poll_all()
        } else {
            looper.poll_all_timeout(LOOPER_TIMEOUT)
        };

        match result {
            Ok(Poll::Event { ident, .. }) => {
                let ident = ident as u32;
                if ident == LOOPER_ID_MAIN {
                    ndk_glue::poll_events()
                } else if ident == LOOPER_ID_INPUT {
                    if let Some(input_queue) = ndk_glue::input_queue().as_ref() {
                        while let Some(event) = input_queue.get_event() {
                            if let Some(event) = input_queue.pre_dispatch(event) {
                                input_queue.finish_event(event, false);
                            }
                        }
                    }
                    None
                } else {
                    unreachable!("Unrecognised looper identifier: {:?}", ident);
                }
            }
            _ => None,
        }
    }

    #[cfg_attr(target_os = "android", ndk_glue::main(backtrace = "on"))]
    fn main() {
        println!("bonjour tout le monde");
        let native_activity = ndk_glue::native_activity();
        let vm_ptr = native_activity.vm();
        let vm: jni::JavaVM = unsafe { jni::JavaVM::from_raw(vm_ptr) }.unwrap();
        let env = vm.attach_current_thread_permanently().unwrap();

        let java: ovr_mobile_sys::ovrJava_ = ovrJava {
            Vm: vm.get_java_vm_pointer(),
            Env: env.get_native_interface(),
            ActivityObject: native_activity.activity(),
        };

        let mut parms: ovrInitParms = ovrInitParms {
            Type: VRAPI_STRUCTURE_TYPE_INIT_PARMS,
            ProductVersion: VRAPI_PRODUCT_VERSION as i32,
            MajorVersion: VRAPI_MAJOR_VERSION as i32,
            MinorVersion: VRAPI_MINOR_VERSION as i32,
            PatchVersion: VRAPI_PATCH_VERSION as i32,
            GraphicsAPI: ovrGraphicsAPI_::VRAPI_GRAPHICS_API_OPENGL_ES_2,
            Java: java,
        };

        let result = unsafe { vrapi_Initialize(&parms) };

        println!("Initialised: {:?}!!", result);

        let mut app = App {
            java,
            ovrMobile: None,
            destroyRequested: false,
            resumed: false,
        };

        while (!app.destroyRequested) {
            loop {
                match poll_all_ms(false) {
                    Some(event) => println!("got an event: {:?}", event),
                    _ => {}
                }
            }
        }
    }
}
