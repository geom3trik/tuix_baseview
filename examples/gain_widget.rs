use baseview::{Size, WindowOpenOptions, WindowScalePolicy};
use tuix_baseview::{RenderSettings, Settings, TuixWindow};

use tuix::events::BuildHandler;
use tuix::widgets::value_knob::*;
use tuix::{Entity, EventHandler, State};

static THEME: &str = include_str!("theme.css");

struct GainWidget {
    control: Entity,
}

impl GainWidget {
    pub fn new() -> Self {
        GainWidget {
            control: Entity::null(),
        }
    }
}

impl BuildHandler for GainWidget {
    type Ret = Entity;
    fn on_build(&mut self, state: &mut State, entity: Entity) -> Self::Ret {
        self.control =
            ValueKnob::new("Gain", 1.0, 0.0, 1.0).build(state, entity, |builder| builder);

        entity
    }
}

impl EventHandler for GainWidget {}

fn main() {
    let settings = Settings {
        window: WindowOpenOptions {
            title: String::from("tuix_baseview gain widget"),
            size: Size::new(300.0, 300.0),
            scale: WindowScalePolicy::SystemScaleFactor,
        },
        clear_color: (0, 0, 0),
        render_settings: RenderSettings::default(),
    };

    TuixWindow::open_blocking(settings, |win_desc, state, window| {
        state.insert_theme(THEME);

        GainWidget::new().build(state, window, |builder| builder);

        win_desc
    });
}
