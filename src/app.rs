use ovr_mobile_sys::{
    ovrJava, ovrMobile, ovrModeFlags, ovrModeParms,
    ovrStructureType_::VRAPI_STRUCTURE_TYPE_MODE_PARMS, vrapi_EnterVrMode,
};
use std::ptr::NonNull;

use crate::vulkan_renderer::VulkanRenderer;
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
        let renderer = unsafe { VulkanRenderer::new() };
        Self {
            java,
            renderer,
            ovr_mobile: None,
            destroy_requested: false,
            resumed: false,
            window_created: false,
        }
    }

    pub fn handle_event(&mut self, event: ndk_glue::Event) -> () {
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
            unsafe {
                self.render();
            }
        }
    }

    fn need_to_enter_vr(&self) -> bool {
        self.resumed && self.window_created && self.ovr_mobile.is_none()
    }

    fn enter_vr(&mut self) {
        println!("[ENTER_VR] Entering VR Mode..");
        let flags = 0u32 | ovrModeFlags::VRAPI_MODE_FLAG_NATIVE_WINDOW as u32;
        let ovr_mode_parms = ovrModeParms {
            Type: VRAPI_STRUCTURE_TYPE_MODE_PARMS,
            Flags: flags,
            Java: self.java.clone(),
            WindowSurface: ndk_glue::native_window().as_ref().unwrap().ptr().as_ptr() as u64,
            Display: 0,
            ShareContext: 0,
        };

        let ovr_mobile = unsafe { vrapi_EnterVrMode(&ovr_mode_parms) };
        println!("[ENTER_VR] Done.");

        self.ovr_mobile = NonNull::new(ovr_mobile);
    }

    fn should_render(&self) -> bool {
        self.resumed && self.window_created && self.ovr_mobile.is_some()
    }

    unsafe fn render(&mut self) {
        let ovr_mobile = self.ovr_mobile.unwrap();
        self.renderer.render(ovr_mobile);
    }
}
