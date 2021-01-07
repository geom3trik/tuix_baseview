use crate::Renderer;
use crate::{event_manager::EventManager, WindowDescription};
use baseview::WindowScalePolicy;
use femtovg::Canvas;
use tuix::events::{Event, Propagation};
use tuix::state::hierarchy::IntoHierarchyIterator;
use tuix::state::mouse::{MouseButton, MouseButtonState};
use tuix::state::Fonts;
use tuix::window::{WindowEvent, WindowWidget};
use tuix::{Entity, State};
use tuix::{Length, Visibility};

pub struct Application {
    state: State,
    event_manager: EventManager,
    canvas: Canvas<Renderer>,
    hierarchy: tuix::Hierarchy,
    pos: (f32, f32),
    should_redraw: bool,
    scale_policy: WindowScalePolicy,
    scale_factor: f32,
    clear_color: femtovg::Color,
    physical_size: (u32, u32),
    _win_description: WindowDescription,
}

impl Application {
    pub fn new<F: FnMut(WindowDescription, &mut State, Entity) -> WindowDescription>(
        mut app: F,
        renderer: Renderer,
        scale_policy: WindowScalePolicy,
        mut logical_size: (u32, u32),
        clear_color: (u8, u8, u8),
        title: String,
    ) -> Self {
        let mut state = State::new();

        let event_manager = EventManager::new();

        let root = state.root;
        state.hierarchy.add(state.root, None);

        let win_description = WindowDescription {
            title,
            inner_size: baseview::Size::new(logical_size.0 as f64, logical_size.1 as f64),
            min_inner_size: baseview::Size::new(100.0, 100.0),
            icon: None,
            icon_width: 0,
            icon_height: 0,
        };
        let win_description = (app)(win_description, &mut state, root);

        logical_size.0 = win_description.inner_size.width.round() as u32;
        logical_size.1 = win_description.inner_size.height.round() as u32;

        // TODO: Request baseview window resize if user has set the size to be different.

        let mut canvas = Canvas::new(renderer).expect("Cannot create canvas");

        let clear_color = femtovg::Color::rgb(clear_color.0, clear_color.1, clear_color.2);

        // Assume scale for now until there is an event with a new one.
        let scale = match scale_policy {
            WindowScalePolicy::ScaleFactor(scale) => scale,
            WindowScalePolicy::SystemScaleFactor => 1.0,
        } as f32;

        let physical_size = (
            (logical_size.0 as f32 * scale).round() as u32,
            (logical_size.1 as f32 * scale).round() as u32,
        );

        canvas.set_size(physical_size.0, physical_size.1, 1.0);

        let regular_font = include_bytes!("../resources/Roboto-Regular.ttf");
        let bold_font = include_bytes!("../resources/Roboto-Bold.ttf");
        let icon_font = include_bytes!("../resources/entypo.ttf");

        let fonts = Fonts {
            regular: Some(canvas.add_font_mem(regular_font).expect("Cannot add font")),
            bold: Some(canvas.add_font_mem(bold_font).expect("Cannot add font")),
            icons: Some(canvas.add_font_mem(icon_font).expect("Cannot add font")),
        };

        state.fonts = fonts;

        canvas.scale(scale, scale);

        state
            .style
            .width
            .insert(state.root, Length::Pixels(logical_size.0 as f32));
        state
            .style
            .height
            .insert(state.root, Length::Pixels(logical_size.1 as f32));

        state
            .transform
            .set_width(state.get_root(), logical_size.0 as f32);
        state
            .transform
            .set_height(state.get_root(), logical_size.1 as f32);
        state.transform.set_opacity(state.get_root(), 1.0);

        WindowWidget::new().build_window(&mut state);

        state.insert_event(Event::new(WindowEvent::Restyle));
        state.insert_event(Event::new(WindowEvent::Relayout).target(Entity::null()));

        let hierarchy = state.hierarchy.clone();

        Application {
            event_manager,
            state,
            canvas,
            hierarchy,
            pos: (0.0, 0.0),
            should_redraw: true,
            scale_policy,
            scale_factor: scale,
            clear_color,
            physical_size,
            _win_description: win_description,
        }
    }

    pub fn get_window(&self) -> Entity {
        self.state.root
    }

    pub fn get_state(&mut self) -> &mut State {
        &mut self.state
    }

    pub fn get_event_manager(&mut self) -> &mut EventManager {
        &mut self.event_manager
    }

    pub fn on_frame_update(&mut self) {
        if self.state.apply_animations() {
            self.state.insert_event(
                Event::new(WindowEvent::Relayout)
                    .target(Entity::null())
                    .origin(Entity::new(0, 0)),
            );
            self.state.insert_event(Event::new(WindowEvent::Redraw));
        }

        while !self.state.event_queue.is_empty() {
            if self.event_manager.flush_events(&mut self.state) {
                self.should_redraw = true;
            }
        }
    }

    pub fn render(&mut self) {
        if self.should_redraw {
            self.event_manager.draw(
                &mut self.state,
                &self.hierarchy,
                &mut self.canvas,
                self.physical_size,
                self.clear_color,
            );
            self.should_redraw = false;
        }
    }

    pub fn handle_event(&mut self, event: baseview::Event, should_quit: &mut bool) {
        if requests_exit(&event) {
            self.state
                .insert_event(Event::new(WindowEvent::WindowClose));
            *should_quit = true;
        }

        match event {
            baseview::Event::Mouse(event) => match event {
                baseview::MouseEvent::CursorMoved { position } => {
                    let cursorx = (position.x) as f32;
                    let cursory = (position.y) as f32;

                    self.state.mouse.cursorx = cursorx;
                    self.state.mouse.cursory = cursory;

                    let mut hovered_widget = Entity::new(0, 0);

                    // This only really needs to be computed when the hierarchy changes
                    // Can be optimised
                    let mut draw_hierarchy: Vec<Entity> =
                        self.state.hierarchy.into_iter().collect();

                    draw_hierarchy
                        .sort_by_cached_key(|entity| self.state.transform.get_z_order(*entity));

                    for widget in draw_hierarchy.into_iter() {
                        // Skip invisible widgets
                        if self.state.transform.get_visibility(widget) == Visibility::Invisible {
                            continue;
                        }

                        // This shouldn't be here but there's a bug if it isn't
                        if self.state.transform.get_opacity(widget) == 0.0 {
                            continue;
                        }

                        // Skip non-hoverable widgets
                        if self.state.transform.get_hoverability(widget) != true {
                            continue;
                        }

                        let border_width = match self
                            .state
                            .style
                            .border_width
                            .get(widget)
                            .cloned()
                            .unwrap_or_default()
                        {
                            Length::Pixels(val) => val,
                            //Length::Percentage(val) => parent_width * val,
                            _ => 0.0,
                        };

                        let posx = self.state.transform.get_posx(widget) - (border_width / 2.0);
                        let posy = self.state.transform.get_posy(widget) - (border_width / 2.0);
                        let width = self.state.transform.get_width(widget) + (border_width);
                        let height = self.state.transform.get_height(widget) + (border_width);

                        let clip_widget = self.state.transform.get_clip_widget(widget);

                        let clip_posx = self.state.transform.get_posx(clip_widget);
                        let clip_posy = self.state.transform.get_posy(clip_widget);
                        let clip_width = self.state.transform.get_width(clip_widget);
                        let clip_height = self.state.transform.get_height(clip_widget);

                        if cursorx >= posx
                            && cursorx >= clip_posx
                            && cursorx < (posx + width)
                            && cursorx < (clip_posx + clip_width)
                            && cursory >= posy
                            && cursory >= clip_posy
                            && cursory < (posy + height)
                            && cursory < (clip_posy + clip_height)
                        {
                            hovered_widget = widget;
                            if let Some(pseudo_classes) =
                                self.state.style.pseudo_classes.get_mut(hovered_widget)
                            {
                                pseudo_classes.set_over(true);
                            }
                        } else {
                            if let Some(pseudo_classes) =
                                self.state.style.pseudo_classes.get_mut(hovered_widget)
                            {
                                pseudo_classes.set_over(false);
                            }
                        }
                    }

                    if hovered_widget != self.state.hovered {
                        // Useful for debugging

                        println!(
                            "Hover changed to {:?} parent: {:?}, posx: {}, posy: {} width: {} height: {} z_order: {}",
                            hovered_widget,
                            self.state.hierarchy.get_parent(hovered_widget),
                            self.state.transform.get_posx(hovered_widget),
                            self.state.transform.get_posy(hovered_widget),
                            self.state.transform.get_width(hovered_widget),
                            self.state.transform.get_height(hovered_widget),
                            self.state.transform.get_z_order(hovered_widget),
                        );

                        if let Some(pseudo_classes) =
                            self.state.style.pseudo_classes.get_mut(hovered_widget)
                        {
                            pseudo_classes.set_hover(true);
                        }

                        if let Some(pseudo_classes) =
                            self.state.style.pseudo_classes.get_mut(self.state.hovered)
                        {
                            pseudo_classes.set_hover(false);
                        }

                        self.state.insert_event(
                            Event::new(WindowEvent::MouseOver).target(hovered_widget),
                        );
                        self.state.insert_event(
                            Event::new(WindowEvent::MouseOut).target(self.state.hovered),
                        );

                        self.state.hovered = hovered_widget;
                        self.state.active = Entity::null();

                        self.state.insert_event(Event::new(WindowEvent::Restyle));
                        self.state.insert_event(Event::new(WindowEvent::Redraw));
                    }

                    if self.state.captured != Entity::null() {
                        self.state.insert_event(
                            Event::new(WindowEvent::MouseMove(cursorx, cursory))
                                .target(self.state.captured)
                                .propagate(Propagation::Direct),
                        );
                    } else if self.state.hovered != Entity::new(0, 0) {
                        self.state.insert_event(
                            Event::new(WindowEvent::MouseMove(cursorx, cursory))
                                .target(self.state.hovered),
                        );
                    }

                    self.pos = (cursorx, cursory);
                }
                baseview::MouseEvent::ButtonPressed(button) => {
                    let b = match button {
                        baseview::MouseButton::Left => MouseButton::Left,
                        baseview::MouseButton::Right => MouseButton::Right,
                        baseview::MouseButton::Middle => MouseButton::Middle,
                        baseview::MouseButton::Other(id) => MouseButton::Other(id as u16),
                        baseview::MouseButton::Back => MouseButton::Other(4),
                        baseview::MouseButton::Forward => MouseButton::Other(5),
                    };

                    match b {
                        MouseButton::Left => {
                            self.state.mouse.left.state = MouseButtonState::Pressed;
                        }
                        MouseButton::Right => {
                            self.state.mouse.right.state = MouseButtonState::Pressed;
                        }
                        MouseButton::Middle => {
                            self.state.mouse.middle.state = MouseButtonState::Pressed;
                        }
                        _ => {}
                    };

                    if self.state.hovered != Entity::null()
                        && self.state.active != self.state.hovered
                    {
                        self.state.active = self.state.hovered;
                        self.state.insert_event(Event::new(WindowEvent::Restyle));
                    }

                    if self.state.captured != Entity::null() {
                        self.state.insert_event(
                            Event::new(WindowEvent::MouseDown(b))
                                .target(self.state.captured)
                                .propagate(Propagation::Direct),
                        );
                    } else {
                        self.state.insert_event(
                            Event::new(WindowEvent::MouseDown(b)).target(self.state.hovered),
                        );
                    }

                    match b {
                        MouseButton::Left => {
                            self.state.mouse.left.pos_down =
                                (self.state.mouse.cursorx, self.state.mouse.cursory);
                            self.state.mouse.left.pressed = self.state.hovered;
                        }

                        MouseButton::Middle => {
                            self.state.mouse.middle.pos_down =
                                (self.state.mouse.cursorx, self.state.mouse.cursory);
                            self.state.mouse.left.pressed = self.state.hovered;
                        }

                        MouseButton::Right => {
                            self.state.mouse.right.pos_down =
                                (self.state.mouse.cursorx, self.state.mouse.cursory);
                            self.state.mouse.left.pressed = self.state.hovered;
                        }

                        _ => {}
                    }
                }
                baseview::MouseEvent::ButtonReleased(button) => {
                    let b = match button {
                        baseview::MouseButton::Left => MouseButton::Left,
                        baseview::MouseButton::Right => MouseButton::Right,
                        baseview::MouseButton::Middle => MouseButton::Middle,
                        baseview::MouseButton::Other(id) => MouseButton::Other(id as u16),
                        baseview::MouseButton::Back => MouseButton::Other(4),
                        baseview::MouseButton::Forward => MouseButton::Other(5),
                    };

                    match b {
                        MouseButton::Left => {
                            self.state.mouse.left.state = MouseButtonState::Released;
                        }
                        MouseButton::Right => {
                            self.state.mouse.right.state = MouseButtonState::Released;
                        }
                        MouseButton::Middle => {
                            self.state.mouse.middle.state = MouseButtonState::Released;
                        }
                        _ => {}
                    };

                    self.state.active = Entity::null();
                    self.state.insert_event(Event::new(WindowEvent::Restyle));

                    if self.state.captured != Entity::null() {
                        self.state.insert_event(
                            Event::new(WindowEvent::MouseUp(b))
                                .target(self.state.captured)
                                .propagate(Propagation::Direct),
                        );
                    } else {
                        self.state.insert_event(
                            Event::new(WindowEvent::MouseUp(b)).target(self.state.hovered),
                        );
                    }

                    match b {
                        MouseButton::Left => {
                            self.state.mouse.left.pos_up =
                                (self.state.mouse.cursorx, self.state.mouse.cursory);
                            self.state.mouse.left.released = self.state.hovered;
                        }

                        MouseButton::Middle => {
                            self.state.mouse.middle.pos_up =
                                (self.state.mouse.cursorx, self.state.mouse.cursory);
                            self.state.mouse.left.released = self.state.hovered;
                        }

                        MouseButton::Right => {
                            self.state.mouse.right.pos_up =
                                (self.state.mouse.cursorx, self.state.mouse.cursory);
                            self.state.mouse.left.released = self.state.hovered;
                        }

                        _ => {}
                    }
                }
                baseview::MouseEvent::WheelScrolled(scroll_delta) => {
                    let (lines_x, lines_y) = match scroll_delta {
                        baseview::ScrollDelta::Lines { x, y } => (x, y),
                        baseview::ScrollDelta::Pixels { x, y } => (
                            if x < 0.0 {
                                -1.0
                            } else if x > 1.0 {
                                1.0
                            } else {
                                0.0
                            },
                            if y < 0.0 {
                                -1.0
                            } else if y > 1.0 {
                                1.0
                            } else {
                                0.0
                            },
                        ),
                    };

                    if self.state.captured != Entity::null() {
                        self.state.insert_event(
                            Event::new(WindowEvent::MouseScroll(lines_x, lines_y))
                                .target(self.state.captured)
                                .propagate(Propagation::Direct),
                        );
                    } else {
                        self.state.insert_event(
                            Event::new(WindowEvent::MouseScroll(lines_x, lines_y))
                                .target(self.state.hovered),
                        );
                    }
                }
                _ => {}
            },
            baseview::Event::Keyboard(event) => {
                use keyboard_types::Code;
                use tuix::PropSet;

                let (s, pressed) = match event.state {
                    keyboard_types::KeyState::Down => (MouseButtonState::Pressed, true),
                    keyboard_types::KeyState::Up => (MouseButtonState::Released, false),
                };

                match event.code {
                    Code::ShiftLeft | Code::ShiftRight => self.state.modifiers.shift = pressed,
                    Code::ControlLeft | Code::ControlRight => self.state.modifiers.ctrl = pressed,
                    Code::AltLeft | Code::AltRight => self.state.modifiers.alt = pressed,
                    Code::MetaLeft | Code::MetaRight => self.state.modifiers.logo = pressed,
                    _ => (),
                }

                if event.code == Code::F5 && s == MouseButtonState::Pressed {
                    println!("Reload Styles");
                    self.state.reload_styles().unwrap();
                }

                if event.code == Code::Tab && s == MouseButtonState::Pressed {
                    let next_focus = self
                        .state
                        .style
                        .focus_order
                        .get(self.state.focused)
                        .cloned()
                        .unwrap_or_default()
                        .next;
                    let prev_focus = self
                        .state
                        .style
                        .focus_order
                        .get(self.state.focused)
                        .cloned()
                        .unwrap_or_default()
                        .prev;

                    if self.state.modifiers.shift {
                        if prev_focus != Entity::null() {
                            self.state.focused.set_focus(&mut self.state, false);
                            self.state.focused = prev_focus;
                            self.state.focused.set_focus(&mut self.state, true);
                        } else {
                            // TODO impliment reverse iterator for hierarchy
                            // state.focused = match state.focused.into_iter(&state.hierarchy).next() {
                            //     Some(val) => val,
                            //     None => state.root,
                            // };
                        }
                    } else {
                        if next_focus != Entity::null() {
                            self.state.focused.set_focus(&mut self.state, false);
                            self.state.focused = next_focus;
                            self.state.focused.set_focus(&mut self.state, true);
                        } else {
                            self.state.focused.set_focus(&mut self.state, false);
                            self.state.focused =
                                match self.state.focused.into_iter(&self.hierarchy).next() {
                                    Some(val) => val,
                                    None => self.state.root,
                                };
                            self.state.focused.set_focus(&mut self.state, true);
                        }
                    }

                    self.state
                        .insert_event(Event::new(WindowEvent::Restyle).target(self.state.root));
                }

                let code = crate::keyboard::baseview_to_tuix_keycode(event.code);

                match s {
                    MouseButtonState::Pressed => {
                        if self.state.focused != Entity::null() {
                            self.state.insert_event(
                                Event::new(WindowEvent::KeyDown(code))
                                    .target(self.state.focused)
                                    .propagate(Propagation::DownUp),
                            );
                        } else {
                            self.state.insert_event(
                                Event::new(WindowEvent::KeyDown(code))
                                    .target(self.state.hovered)
                                    .propagate(Propagation::DownUp),
                            );
                        }

                        if let keyboard_types::Key::Character(written) = &event.key {
                            for chr in written.chars() {
                                self.state.insert_event(
                                    Event::new(WindowEvent::CharInput(chr))
                                        .target(self.state.focused)
                                        .propagate(Propagation::Down),
                                );
                            }
                        }
                    }

                    MouseButtonState::Released => {
                        if self.state.focused != Entity::null() {
                            self.state.insert_event(
                                Event::new(WindowEvent::KeyUp(code))
                                    .target(self.state.focused)
                                    .propagate(Propagation::DownUp),
                            );
                        } else {
                            self.state.insert_event(
                                Event::new(WindowEvent::KeyUp(code))
                                    .target(self.state.hovered)
                                    .propagate(Propagation::DownUp),
                            );
                        }
                    }
                }
            }
            baseview::Event::Window(event) => match event {
                baseview::WindowEvent::Focused => {
                    self.state
                        .insert_event(Event::new(WindowEvent::Restyle).target(self.state.root));
                }
                baseview::WindowEvent::Resized(window_info) => {
                    self.scale_factor = match self.scale_policy {
                        WindowScalePolicy::ScaleFactor(scale) => scale,
                        WindowScalePolicy::SystemScaleFactor => window_info.scale(),
                    } as f32;

                    let logical_size = (
                        (window_info.physical_size().width as f32 / self.scale_factor),
                        (window_info.physical_size().height as f32 / self.scale_factor),
                    );

                    self.physical_size = (
                        window_info.physical_size().width,
                        window_info.physical_size().height,
                    );

                    self.canvas.reset_transform();
                    self.canvas
                        .set_size(self.physical_size.0, self.physical_size.1, 1.0);
                    self.canvas.scale(self.scale_factor, self.scale_factor);

                    self.state
                        .style
                        .width
                        .insert(self.state.root, Length::Pixels(logical_size.0));
                    self.state
                        .style
                        .height
                        .insert(self.state.root, Length::Pixels(logical_size.1));

                    self.state
                        .transform
                        .set_width(self.state.root, logical_size.0);
                    self.state
                        .transform
                        .set_height(self.state.root, logical_size.1);

                    self.state.insert_event(Event::new(WindowEvent::Restyle));
                    self.state
                        .insert_event(Event::new(WindowEvent::Relayout).target(Entity::null()));
                    self.state.insert_event(Event::new(WindowEvent::Redraw));
                }
                baseview::WindowEvent::WillClose => {
                    self.state
                        .insert_event(Event::new(WindowEvent::WindowClose));
                }
                _ => {}
            },
        }
    }
}

/// Returns true if the provided event should cause an [`Application`] to
/// exit.
pub fn requests_exit(event: &baseview::Event) -> bool {
    match event {
        baseview::Event::Window(baseview::WindowEvent::WillClose) => true,
        #[cfg(target_os = "macos")]
        baseview::Event::Keyboard(event) => {
            if event.code == keyboard_types::Code::KeyQ {
                if event.modifiers == keyboard_types::Modifiers::META {
                    if event.state == keyboard_types::KeyState::Down {
                        return true;
                    }
                }
            }

            false
        }
        _ => false,
    }
}
