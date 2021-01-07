use crate::{settings, Application, Renderer, Settings, WindowDescription};
use baseview::{Event, Window, WindowHandler, WindowOpenOptions, WindowScalePolicy};
use raw_window_handle::{HasRawWindowHandle, RawWindowHandle};
use tuix::{Entity, State};

/// Handles an tuix_baseview application
pub struct TuixWindow {
    application: Application,

    #[cfg(feature = "opengl")]
    context: raw_gl_context::GlContext,
}

impl TuixWindow {
    fn new<F>(app: F, window: &mut baseview::Window<'_>, settings: Settings) -> TuixWindow
    where
        F: FnMut(WindowDescription, &mut State, Entity) -> WindowDescription,
        F: 'static + Send,
    {
        // WindowScalePolicy does not implement Copy/Clone.
        let scale_policy = match &settings.window.scale {
            WindowScalePolicy::SystemScaleFactor => WindowScalePolicy::SystemScaleFactor,
            WindowScalePolicy::ScaleFactor(scale) => WindowScalePolicy::ScaleFactor(*scale),
        };
        let logical_size = (
            settings.window.size.width.round() as u32,
            settings.window.size.height.round() as u32,
        );

        let (renderer, context) = load_renderer(window, settings.render_settings);

        let application = Application::new(
            app,
            renderer,
            scale_policy,
            logical_size,
            settings.clear_color,
            settings.window.title,
        );

        TuixWindow {
            application,
            context,
        }
    }

    /// Open a new child window.
    ///
    /// * `parent` - The parent window.
    /// * `settings` - The settings of the window.
    /// * `app` - The Tuix application builder.
    pub fn open_parented<P, F>(parent: &P, settings: Settings, app: F)
    where
        P: HasRawWindowHandle,
        F: FnMut(WindowDescription, &mut State, Entity) -> WindowDescription,
        F: 'static + Send,
    {
        // WindowScalePolicy does not implement Copy/Clone.
        let window_scale_policy = match &settings.window.scale {
            WindowScalePolicy::ScaleFactor(scale) => WindowScalePolicy::ScaleFactor(*scale),
            WindowScalePolicy::SystemScaleFactor => WindowScalePolicy::SystemScaleFactor,
        };
        // WindowOpenOptions does not implement Copy/Clone.
        let window_settings = WindowOpenOptions {
            title: settings.window.title.clone(),
            size: settings.window.size,
            scale: window_scale_policy,
        };

        Window::open_parented(
            parent,
            window_settings,
            move |window: &mut baseview::Window<'_>| -> TuixWindow {
                TuixWindow::new(app, window, settings)
            },
        )
    }

    /// Open a new window as if it had a parent window.
    ///
    /// * `settings` - The settings of the window.
    /// * `app` - The Tuix application builder.
    pub fn open_as_if_parented<F>(settings: Settings, app: F) -> RawWindowHandle
    where
        F: FnMut(WindowDescription, &mut State, Entity) -> WindowDescription,
        F: 'static + Send,
    {
        // WindowScalePolicy does not implement Copy/Clone.
        let window_scale_policy = match &settings.window.scale {
            WindowScalePolicy::ScaleFactor(scale) => WindowScalePolicy::ScaleFactor(*scale),
            WindowScalePolicy::SystemScaleFactor => WindowScalePolicy::SystemScaleFactor,
        };
        // WindowOpenOptions does not implement Copy/Clone.
        let window_settings = WindowOpenOptions {
            title: settings.window.title.clone(),
            size: settings.window.size,
            scale: window_scale_policy,
        };

        Window::open_as_if_parented(
            window_settings,
            move |window: &mut baseview::Window<'_>| -> TuixWindow {
                TuixWindow::new(app, window, settings)
            },
        )
    }

    /// Open a new window that blocks the current thread until the window is destroyed.
    ///
    /// * `settings` - The settings of the window.
    /// * `app` - The Tuix application builder.
    pub fn open_blocking<F>(settings: Settings, app: F)
    where
        F: FnMut(WindowDescription, &mut State, Entity) -> WindowDescription,
        F: 'static + Send,
    {
        // WindowScalePolicy does not implement Copy/Clone.
        let window_scale_policy = match &settings.window.scale {
            WindowScalePolicy::ScaleFactor(scale) => WindowScalePolicy::ScaleFactor(*scale),
            WindowScalePolicy::SystemScaleFactor => WindowScalePolicy::SystemScaleFactor,
        };
        // WindowOpenOptions does not implement Copy/Clone.
        let window_settings = WindowOpenOptions {
            title: settings.window.title.clone(),
            size: settings.window.size,
            scale: window_scale_policy,
        };

        Window::open_blocking(
            window_settings,
            move |window: &mut baseview::Window<'_>| -> TuixWindow {
                TuixWindow::new(app, window, settings)
            },
        )
    }
}

impl WindowHandler for TuixWindow {
    fn on_frame(&mut self) {
        self.application.on_frame_update();

        self.context.make_current();

        self.application.render();

        self.context.swap_buffers();
        self.context.make_not_current();
    }

    fn on_event(&mut self, _window: &mut Window<'_>, event: Event) {
        let mut should_quit = false;
        self.application.handle_event(event, &mut should_quit);

        if should_quit {
            // TODO: Request close.
        }
    }
}

#[cfg(feature = "opengl")]
fn load_renderer(
    window: &Window,
    render_settings: settings::RenderSettings,
) -> (Renderer, raw_gl_context::GlContext) {
    let context = raw_gl_context::GlContext::create(window, render_settings).unwrap();

    context.make_current();

    gl::load_with(|s| context.get_proc_address(s) as _);

    let renderer = femtovg::renderer::OpenGl::new(|s| context.get_proc_address(s) as *const _)
        .expect("Cannot create renderer");

    context.make_not_current();

    (renderer, context)
}
