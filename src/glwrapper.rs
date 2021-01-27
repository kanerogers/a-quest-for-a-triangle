use egli::{egl, Context, Display, RenderableType, SurfaceType};

pub struct GLWrapper {
    pub display: *mut std::ffi::c_void,
    pub surface: *mut std::ffi::c_void,
    pub context: Context,
}

impl GLWrapper {
    pub fn new() -> GLWrapper {
        let display = Display::from_default_display().expect("Unable to get default display");
        let configs = display
            .config_filter()
            .with_red_size(8)
            .with_green_size(8)
            .with_blue_size(8)
            .with_depth_size(24)
            .with_surface_type(SurfaceType::WINDOW)
            .with_renderable_type(RenderableType::OPENGL)
            .choose_configs()
            .expect("failed to get configurations");
        let first_config = *configs.first().expect("No configurations found");
        let context = display
            .create_context(first_config)
            .expect("Unable to create context");

        let display = display.forget();
        let first_config = first_config.handle();
        let attrib_list = [egl::EGL_WIDTH, 16, egl::EGL_HEIGHT, 16, egl::EGL_NONE];

        let surface = egl::create_pbuffer_surface(display, first_config, &attrib_list)
            .expect("Unable to create surface");
        egl::make_current(display, surface, surface, context.handle())
            .expect("Unable to make current");

        GLWrapper {
            display,
            context,
            surface,
        }
    }
}
