use crate::glwrapper::GLWrapper;
use ovr_mobile_sys::{
    helpers::vrapi_DefaultLayerLoadingIcon2, ovrJava, ovrLayerHeader2, ovrMobile, ovrModeFlags,
    ovrModeParms, ovrStructureType_::VRAPI_STRUCTURE_TYPE_MODE_PARMS, ovrSubmitFrameDescription2_,
    ovrTextureSwapChain, vrapi_EnterVrMode, vrapi_GetPredictedDisplayTime,
    vrapi_GetPredictedTracking2, vrapi_SubmitFrame2,
};

pub struct App {
    pub java: ovrJava,
    pub ovr_mobile: Option<*mut ovrMobile>,
    pub destroy_requested: bool,
    pub resumed: bool,
    pub window_created: bool,
    pub gl: GLWrapper,
    pub frame_index: i64,
    pub color_texture_swap_chain: [*mut ovrTextureSwapChain; 2],
}

impl App {
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
        // if self.need_to_exit_vr() {
        //     self.exit_vr();
        // }

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
        let flags = 0u32 | ovrModeFlags::VRAPI_MODE_FLAG_NATIVE_WINDOW as u32;
        let ovr_mode_parms = ovrModeParms {
            Type: VRAPI_STRUCTURE_TYPE_MODE_PARMS,
            Flags: flags,
            Java: self.java.clone(),
            WindowSurface: ndk_glue::native_window().as_ref().unwrap().ptr().as_ptr() as u64,
            Display: self.gl.display as u64,
            ShareContext: self.gl.context as u64,
        };

        println!("[ENTER_VR] Entering VR Mode..");
        let ovr_mobile = unsafe { vrapi_EnterVrMode(&ovr_mode_parms) };
        println!("[ENTER_VR] Done.");

        self.ovr_mobile = Some(ovr_mobile);
    }

    fn should_render(&self) -> bool {
        self.resumed && self.window_created && self.ovr_mobile.is_some()
    }

    unsafe fn render(&mut self) {
        // Get the HMD pose, predicted for the middle of the time period during which
        // the new eye images will be displayed. The number of frames predicted ahead
        // depends on the pipeline depth of the engine and the synthesis rate.
        // The better the prediction, the less black will be pulled in at the edges.
        let predicted_display_time =
            vrapi_GetPredictedDisplayTime(*self.ovr_mobile.as_ref().unwrap(), self.frame_index);
        let _tracking =
            vrapi_GetPredictedTracking2(*self.ovr_mobile.as_ref().unwrap(), predicted_display_time);

        // Advance the simulation based on the predicted display time.

        // Render eye images and setup the 'ovrSubmitFrameDescription2' using 'ovrTracking2' data.

        let layer = vrapi_DefaultLayerLoadingIcon2();
        // layer.HeadPose = tracking.HeadPose;
        // for eye in 0..2 {
        //     let colorTextureSwapChainIndex = self.frame_index as i32
        //         % vrapi_GetTextureSwapChainLength(self.color_texture_swap_chain[eye]);
        //     let textureId = vrapi_GetTextureSwapChainHandle(
        //         self.color_texture_swap_chain[eye],
        //         colorTextureSwapChainIndex,
        //     );
        //     //     // Render to 'textureId' using the 'ProjectionMatrix' from 'ovrTracking2'.

        //     layer.Textures[eye].ColorSwapChain = self.color_texture_swap_chain[eye];
        //     layer.Textures[eye].SwapChainIndex = colorTextureSwapChainIndex;
        //     layer.Textures[eye].TexCoordsFromTanAngles =
        //         ovrMatrix4f_TanAngleMatrixFromProjection(&tracking.Eye[eye].ProjectionMatrix);
        // }

        let layers = [&layer.Header as *const ovrLayerHeader2];

        let frame_desc = ovrSubmitFrameDescription2_ {
            Flags: 0,
            SwapInterval: 1,
            FrameIndex: self.frame_index as u64,
            Pad: std::mem::zeroed(),
            DisplayTime: predicted_display_time,
            LayerCount: 1,
            Layers: layers.as_ptr(),
        };

        // Hand over the eye images to the time warp.
        vrapi_SubmitFrame2(*self.ovr_mobile.as_ref().unwrap(), &frame_desc);
    }
}