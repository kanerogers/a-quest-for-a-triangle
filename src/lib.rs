mod glwrapper;

#[allow(non_snake_case)]
#[cfg(target_os = "android")]
mod lib {
    use crate::glwrapper::GLWrapper;
    use ndk::looper::{Poll, ThreadLooper};
    use ovr_mobile_sys::ovrStructureType_::{
        VRAPI_STRUCTURE_TYPE_INIT_PARMS, VRAPI_STRUCTURE_TYPE_MODE_PARMS,
    };
    use ovr_mobile_sys::ovrSystemProperty_::{
        VRAPI_SYS_PROP_SUGGESTED_EYE_TEXTURE_HEIGHT, VRAPI_SYS_PROP_SUGGESTED_EYE_TEXTURE_WIDTH,
    };
    use ovr_mobile_sys::ovrTextureType_::VRAPI_TEXTURE_TYPE_2D;
    use ovr_mobile_sys::VRAPI_PRODUCT_VERSION;
    use ovr_mobile_sys::{ovrGraphicsAPI_, ovrInitParms};
    use ovr_mobile_sys::{ovrJava, ovrMobile, ovrModeFlags, ovrModeParms, ovrTextureSwapChain};
    use ovr_mobile_sys::{
        vrapi_CreateTextureSwapChain3, vrapi_EnterVrMode, vrapi_GetSystemPropertyInt,
        vrapi_Initialize,
    };
    use ovr_mobile_sys::{VRAPI_MAJOR_VERSION, VRAPI_MINOR_VERSION, VRAPI_PATCH_VERSION};
    use std::time::Duration;

    pub const LOOPER_ID_MAIN: u32 = 0;
    pub const LOOPER_ID_INPUT: u32 = 1;
    pub const LOOPER_TIMEOUT: Duration = Duration::from_millis(0u64);

    struct App {
        java: ovrJava,
        ovrMobile: Option<*mut ovrMobile>,
        destroy_requested: bool,
        resumed: bool,
        window_created: bool,
        gl: GLWrapper,
        frame_index: u64,
        color_texture_swap_chain: [*mut ovrTextureSwapChain; 2],
    }

    impl App {
        fn handle_event(&mut self, event: ndk_glue::Event) -> () {
            println!("[EVENT] Received event: {:?}", event);
            match event {
                ndk_glue::Event::Resume => self.resumed = true,
                ndk_glue::Event::Destroy => self.destroy_requested = true,
                ndk_glue::Event::WindowCreated => self.window_created = true,
                ndk_glue::Event::WindowDestroyed => self.window_created = false,
                ndk_glue::Event::Pause => self.resumed = false,
                _ => {}
            }

            self.next_state();
        }

        fn next_state(&mut self) {
            if self.need_to_enter_vr() {
                self.enter_vr();
            }

            if self.should_render() {
                self.render();
            }

            // if self.need_to_exit_vr() {
            //     self.exit_vr();
            // }
        }

        fn need_to_enter_vr(&self) -> bool {
            self.resumed && self.window_created && self.ovrMobile.is_none()
        }

        fn enter_vr(&mut self) {
            let flags = 0u32 | ovrModeFlags::VRAPI_MODE_FLAG_NATIVE_WINDOW as u32;
            let ovrModeParms = ovrModeParms {
                Type: VRAPI_STRUCTURE_TYPE_MODE_PARMS,
                Flags: flags,
                Java: self.java.clone(),
                WindowSurface: ndk_glue::native_window().as_ref().unwrap().ptr().as_ptr() as u64,
                Display: self.gl.display as u64,
                ShareContext: self.gl.context as u64,
            };

            println!("[ENTER_VR] Entering VR Mode..");
            let ovrMobile = unsafe { vrapi_EnterVrMode(&ovrModeParms) };
            println!("[ENTER_VR] Done.");

            self.ovrMobile = Some(ovrMobile);
        }

        fn render(&mut self) {}
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
                    unreachable!(
                        "Unrecognised looper identifier: {:?} but LOOPER_ID_INPUT is {:?}",
                        ident, LOOPER_ID_INPUT
                    );
                }
            }
            _ => None,
        }
    }

    #[cfg_attr(target_os = "android", ndk_glue::main(backtrace = "on"))]
    fn main() {
        println!("[INIT] Main called");
        let native_activity = ndk_glue::native_activity();
        let vm_ptr = native_activity.vm();

        let vm: jni::JavaVM = unsafe { jni::JavaVM::from_raw(vm_ptr) }.unwrap();
        let env = vm.attach_current_thread_permanently().unwrap();

        let java: ovr_mobile_sys::ovrJava_ = ovrJava {
            Vm: vm.get_java_vm_pointer(),
            Env: env.get_native_interface(),
            ActivityObject: native_activity.activity(),
        };

        let parms: ovrInitParms = ovrInitParms {
            Type: VRAPI_STRUCTURE_TYPE_INIT_PARMS,
            ProductVersion: VRAPI_PRODUCT_VERSION as i32,
            MajorVersion: VRAPI_MAJOR_VERSION as i32,
            MinorVersion: VRAPI_MINOR_VERSION as i32,
            PatchVersion: VRAPI_PATCH_VERSION as i32,
            GraphicsAPI: ovrGraphicsAPI_::VRAPI_GRAPHICS_API_OPENGL_ES_2,
            Java: java,
        };

        println!("[INIT] Initialising vrapi");

        let result = unsafe { vrapi_Initialize(&parms) };

        println!("[INIT] vrapi_Initialize Result: {:?}", result);
        let suggested_eye_texture_width = unsafe {
            vrapi_GetSystemPropertyInt(&java, VRAPI_SYS_PROP_SUGGESTED_EYE_TEXTURE_WIDTH)
        };
        let suggested_eye_texture_height = unsafe {
            vrapi_GetSystemPropertyInt(&java, VRAPI_SYS_PROP_SUGGESTED_EYE_TEXTURE_HEIGHT)
        };

        let color_texture_swap_chain = unsafe {
            [
                vrapi_CreateTextureSwapChain3(
                    VRAPI_TEXTURE_TYPE_2D,
                    gl::RGBA8.into(),
                    suggested_eye_texture_width,
                    suggested_eye_texture_height,
                    1,
                    3,
                ),
                vrapi_CreateTextureSwapChain3(
                    VRAPI_TEXTURE_TYPE_2D,
                    gl::RGBA8.into(),
                    suggested_eye_texture_width,
                    suggested_eye_texture_height,
                    1,
                    3,
                ),
            ]
        };

        let mut app = App {
            java,
            ovrMobile: None,
            destroy_requested: false,
            resumed: false,
            window_created: false,
            gl: GLWrapper::new(),
            frame_index: 1,
            color_texture_swap_chain,
        };

        println!("[INIT] Beginning loop..");

        while !app.destroy_requested {
            loop {
                match poll_all_ms(false) {
                    Some(event) => app.handle_event(event),
                    _ => break,
                }
            }
        }

        println!("Destroy requested! Bye for now!");
    }
}
