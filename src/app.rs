use ash::vk::Handle;
use ndk::looper::{Poll, ThreadLooper};
use ovr_mobile_sys::{
    ovrEventDataBuffer, ovrEventHeader_, ovrEventType, ovrJava, ovrMobile, ovrModeFlags,
    ovrModeParms, ovrModeParmsVulkan, ovrStructureType_::VRAPI_STRUCTURE_TYPE_MODE_PARMS_VULKAN,
    ovrSuccessResult_, vrapi_DestroySystemVulkan, vrapi_EnterVrMode, vrapi_LeaveVrMode,
    vrapi_PollEvent, vrapi_Shutdown,
};
use std::mem::MaybeUninit;
use std::ptr::NonNull;
use std::time::Duration;

use crate::vulkan_renderer::VulkanRenderer;

pub const LOOPER_ID_MAIN: u32 = 0;
pub const LOOPER_ID_INPUT: u32 = 1;
pub const LOOPER_TIMEOUT: Duration = Duration::from_millis(0u64);
pub struct App {
    pub java: ovrJava,
    pub destroy_requested: bool,
    pub resumed: bool,
    pub window_created: bool,
    pub renderer: VulkanRenderer,
    pub ovr_mobile: Option<NonNull<ovrMobile>>,
}

impl App {
    pub fn new(java: ovrJava) -> Self {
        let renderer = unsafe { VulkanRenderer::new(&java) };
        Self {
            java,
            renderer,
            ovr_mobile: None,
            destroy_requested: false,
            resumed: false,
            window_created: false,
        }
    }

    pub fn run(&mut self) {
        while !self.destroy_requested {
            loop {
                match self.poll_android_events() {
                    Some(event) => self.handle_android_event(event),
                    _ => break,
                }
            }
            loop {
                match self.poll_vr_api_events() {
                    Some(e) => self.handle_vr_api_event(e),
                    _ => break,
                }
            }
            self.next_state();
        }
    }

    pub fn handle_vr_api_event(&mut self, event: ovrEventType) -> () {
        println!("[VR_API_EVENTS] Received VR event {:?}", event);
        match event {
            ovrEventType::VRAPI_EVENT_DATA_LOST => {}
            ovrEventType::VRAPI_EVENT_NONE => {}
            ovrEventType::VRAPI_EVENT_VISIBILITY_GAINED => {}
            ovrEventType::VRAPI_EVENT_VISIBILITY_LOST => {}
            ovrEventType::VRAPI_EVENT_FOCUS_GAINED => {}
            ovrEventType::VRAPI_EVENT_FOCUS_LOST => {}
            ovrEventType::VRAPI_EVENT_DISPLAY_REFRESH_RATE_CHANGE => {}
        }
    }

    pub fn handle_android_event(&mut self, event: ndk_glue::Event) -> () {
        println!("[ANDROID_EVENT] Received event: {:?}", event);
        match event {
            ndk_glue::Event::Resume => self.resumed = true,
            ndk_glue::Event::Destroy => self.destroy_requested = true,
            ndk_glue::Event::WindowCreated => self.window_created = true,
            ndk_glue::Event::WindowDestroyed => self.window_created = false,
            ndk_glue::Event::Pause => self.resumed = false,
            _ => {}
        }
    }

    fn next_state(&mut self) {
        if self.need_to_exit_vr() {
            unsafe { self.exit_vr() };
            return;
        }
        if self.need_to_enter_vr() {
            self.enter_vr();
            return;
        }
        if self.should_render() {
            unsafe { self.render() };
            return;
        }
        if self.destroy_requested {
            unsafe { self.destroy() };
            return;
        }
    }

    fn need_to_exit_vr(&self) -> bool {
        if self.ovr_mobile.is_none() {
            return false;
        };
        !self.resumed || !self.window_created
    }

    fn need_to_enter_vr(&self) -> bool {
        if self.ovr_mobile.is_some() {
            return false;
        };
        self.resumed && self.window_created
    }

    fn enter_vr(&mut self) {
        println!("[App] Entering VR Mode..");
        let flags = 0u32 | ovrModeFlags::VRAPI_MODE_FLAG_NATIVE_WINDOW as u32;
        let mode_parms = ovrModeParms {
            Type: VRAPI_STRUCTURE_TYPE_MODE_PARMS_VULKAN,
            Flags: flags,
            Java: self.java.clone(),
            WindowSurface: ndk_glue::native_window().as_ref().unwrap().ptr().as_ptr() as u64,
            Display: 0,
            ShareContext: 0,
        };
        let queue = self.renderer.context.graphics_queue.as_raw();
        let mut parms = ovrModeParmsVulkan {
            ModeParms: mode_parms,
            SynchronizationQueue: queue,
        };
        let parms = NonNull::new(&mut parms).unwrap();

        let ovr_mobile = unsafe { vrapi_EnterVrMode(parms.as_ptr() as *const ovrModeParms) };
        println!("[App] Done. Preparing for first render..");

        self.ovr_mobile = NonNull::new(ovr_mobile);
    }

    unsafe fn destroy(&mut self) {
        println!("[App] Destroying app..");
        vrapi_DestroySystemVulkan();
        vrapi_Shutdown();
        println!("[App] ..done");
    }

    unsafe fn exit_vr(&mut self) {
        println!("[App] Exiting VR mode..");
        let ovr_mobile = self.ovr_mobile.take().unwrap();
        vrapi_LeaveVrMode(ovr_mobile.as_ptr());
        println!("[App] ..done");
    }

    fn should_render(&self) -> bool {
        !self.destroy_requested && self.resumed && self.window_created && self.ovr_mobile.is_some()
    }

    unsafe fn render(&mut self) {
        let ovr_mobile = self.ovr_mobile.unwrap();
        self.renderer.render(ovr_mobile);
    }

    pub fn poll_android_events(&mut self) -> Option<ndk_glue::Event> {
        let looper = ThreadLooper::for_thread().unwrap();
        let result = looper.poll_all_timeout(LOOPER_TIMEOUT);

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

    pub fn poll_vr_api_events(&mut self) -> Option<ovrEventType> {
        let data = unsafe { MaybeUninit::uninit().assume_init() };
        let mut header = ovrEventHeader_ {
            EventType: ovrEventType::VRAPI_EVENT_NONE,
        };

        let _event_data_buffer = ovrEventDataBuffer {
            EventHeader: header,
            EventData: data,
        };

        let pointer = NonNull::new(&mut header).unwrap();

        let result = unsafe { vrapi_PollEvent(pointer.as_ptr()) };
        if result != ovrSuccessResult_::ovrSuccess as i32 {
            return None;
        }

        if header.EventType == ovrEventType::VRAPI_EVENT_NONE {
            return None;
        }

        return Some(header.EventType);
    }
}
