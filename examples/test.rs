extern crate tuix;


use tuix::widgets::Button;
use tuix_baseview::Application;

use tuix::events::BuildHandler;

use tuix::style::themes::DEFAULT_THEME;

fn main() {
    let mut app = Application::new(|win_desc, state, window| {

        state.insert_theme(DEFAULT_THEME);

        Button::new().build(state, window, |builder| {
            builder.set_text("Button")
        });

        win_desc.with_title("Hello GUI")
    });
}