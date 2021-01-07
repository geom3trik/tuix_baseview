mod application;
mod event_manager;
mod keyboard;
mod settings;
mod window;

pub use application::Application;
pub use settings::{RenderSettings, Settings};
pub use window::TuixWindow;

#[cfg(feature = "opengl")]
use femtovg::renderer::OpenGl as Renderer;

pub struct WindowDescription {
    pub title: String,
    pub inner_size: baseview::Size,
    pub min_inner_size: baseview::Size,
    // Change this to resource id when the resource manager is working
    pub icon: Option<Vec<u8>>,
    pub icon_width: u32,
    pub icon_height: u32,
}

impl WindowDescription {
    pub fn new() -> Self {
        WindowDescription {
            title: "Default".to_string(),
            inner_size: baseview::Size::new(800.0, 600.0),
            min_inner_size: baseview::Size::new(100.0, 100.0),
            icon: None,
            icon_width: 0,
            icon_height: 0,
        }
    }

    pub fn with_title(mut self, title: &str) -> Self {
        self.title = title.to_string();

        self
    }

    pub fn with_inner_size(mut self, width: u32, height: u32) -> Self {
        self.inner_size = baseview::Size::new(width as f64, height as f64);

        self
    }

    pub fn with_min_inner_size(mut self, width: u32, height: u32) -> Self {
        self.min_inner_size = baseview::Size::new(width as f64, height as f64);

        self
    }

    pub fn with_icon(mut self, icon: Vec<u8>, width: u32, height: u32) -> Self {
        self.icon = Some(icon);
        self.icon_width = width;
        self.icon_height = height;
        self
    }
}
