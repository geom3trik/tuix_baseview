//! Configure your application;

use baseview::WindowOpenOptions;

// Re-export this in case tuix ever gets another rendering backend such as wgpu.
#[cfg(feature = "opengl")]
pub use raw_gl_context::GlConfig as RenderSettings;

/// The settings of an application.
pub struct Settings {
    /// The `baseview` window settings.
    pub window: WindowOpenOptions,

    /// The settings for the rendering backend.
    pub render_settings: RenderSettings,

    /// The clear color.
    pub clear_color: (u8, u8, u8),
}
