mod app;
mod colour_swap_chain;
mod render_pass;
mod vulkan_renderer;

#[allow(non_snake_case)]
mod lib {
    use crate::app::App;
    use crate::vulkan_renderer::VulkanRenderer;

    use ndk::looper::{Poll, ThreadLooper};
    use ovr_mobile_sys::{
        ovrGraphicsAPI_, ovrInitParms, ovrJava, ovrResult,
        ovrStructureType_::VRAPI_STRUCTURE_TYPE_INIT_PARMS, ovrSystemCreateInfoVulkan,
        vrapi_CreateSystemVulkan, vrapi_Initialize, VRAPI_MAJOR_VERSION, VRAPI_MINOR_VERSION,
        VRAPI_PATCH_VERSION, VRAPI_PRODUCT_VERSION,
    };

    use std::time::Duration;

    pub const LOOPER_ID_MAIN: u32 = 0;
    pub const LOOPER_ID_INPUT: u32 = 1;
    pub const LOOPER_TIMEOUT: Duration = Duration::from_millis(0u64);

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

        let initOvrResult = init_ovr(java);
        println!("[INIT] vrapi_Initialize Result: {:?}", initOvrResult);

        // Create Vulkan Context.
        let renderer = VulkanRenderer::new(&java);

        let init_vulkan_result = init_vulkan(&renderer);

        let mut app = App {
            java,
            ovr_mobile: None,
            destroy_requested: false,
            resumed: false,
            window_created: false,
            renderer,
            frame_index: 1,
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

    fn init_vulkan(vulkan_renderer: &VulkanRenderer) -> ovrResult {
        let system_info = ovrSystemCreateInfoVulkan {
            Instance: todo!(),
            Device: todo!(),
            PhysicalDevice: todo!(),
        };
        unsafe { vrapi_CreateSystemVulkan(&mut system_info) }
    }

    fn init_ovr(java: ovr_mobile_sys::ovrJava_) -> ovr_mobile_sys::ovrInitializeStatus_ {
        let parms: ovrInitParms = ovrInitParms {
            Type: VRAPI_STRUCTURE_TYPE_INIT_PARMS,
            ProductVersion: VRAPI_PRODUCT_VERSION as i32,
            MajorVersion: VRAPI_MAJOR_VERSION as i32,
            MinorVersion: VRAPI_MINOR_VERSION as i32,
            PatchVersion: VRAPI_PATCH_VERSION as i32,
            GraphicsAPI: ovrGraphicsAPI_::VRAPI_GRAPHICS_API_VULKAN_1,
            Java: java,
        };
        println!("[INIT] Initialising vrapi");
        let result = unsafe { vrapi_Initialize(&parms) };
        result
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
}
