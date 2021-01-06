

use tuix_baseview::Application;

use tuix::{Entity, Event, State, BuildHandler, EventHandler};

use tuix::style::{Length, Color};

use tuix::widgets::value_knob::*;
use tuix::widgets::control_knob::*;

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
        
        self.control = ValueKnob::new("Gain", 1.0, 0.0, 1.0).build(state, entity, |builder| builder);
        
        entity
    }
}

impl EventHandler for GainWidget {

}

fn main() {
    let mut app = Application::new(|win_desc, state, window| {

        state.insert_theme(THEME);

        GainWidget::new().build(state, window, |builder| builder);

        win_desc.with_title("Hello GUI").with_inner_size(300,300)
    });
}