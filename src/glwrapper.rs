use egli::{egl, Context, Display, SurfaceType};
use gl;
use std::mem;

pub struct GLWrapper {
    pub display: *mut std::ffi::c_void,
    pub surface: *mut std::ffi::c_void,
    pub context: *mut std::ffi::c_void,
}

impl GLWrapper {
    pub fn new() -> GLWrapper {
        println!("[GL] Creating GL Wrapper..");
        println!("[GL] Getting display..");
        let display = Display::from_default_display().expect("Unable to get default display");
        println!("[GL] Done!");

        println!("[GL] Geting configs..");
        let configs = display
            .config_filter()
            .with_red_size(8)
            .with_green_size(8)
            .with_blue_size(8)
            .with_alpha_size(8)
            .with_depth_size(0)
            .with_stencil_size(0)
            .with_surface_type(SurfaceType::PBUFFER | SurfaceType::WINDOW)
            .choose_configs()
            .expect("failed to get configurations");
        println!("[GL] Done!");
        let first_config = *configs.first().expect("No configurations found");
        let context = display
            .create_context(first_config)
            .expect("Unable to create context")
            .forget();

        let display = display.forget();
        let first_config = first_config.handle();
        let attrib_list = [egl::EGL_WIDTH, 16, egl::EGL_HEIGHT, 16, egl::EGL_NONE];

        let surface = egl::create_pbuffer_surface(display, first_config, &attrib_list)
            .expect("Unable to create surface");
        egl::make_current(display, surface, surface, context).expect("Unable to make current");

        gl::load_with(|s| unsafe { mem::transmute(egl::get_proc_address(s)) });

        GLWrapper {
            display,
            context,
            surface,
        }
    }
}
