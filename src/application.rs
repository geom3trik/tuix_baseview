




#![allow(deprecated)]

use raw_window_handle::{HasRawWindowHandle};

use tuix::window::{WindowDescription, WindowEvent, WindowWidget};

use tuix::{Entity, State};
use tuix::{Length, Visibility};

use tuix::state::mouse::{MouseButton, MouseButtonState};

use tuix::events::{Event, Propagation};

use tuix::state::Fonts;

use crate::event_manager::EventManager;

use femtovg::{
    renderer::OpenGl,
    Canvas,
    Color,
};

use baseview::{WindowHandler, WindowScalePolicy};

use raw_gl_context::GlContext;

struct OpenWindowExample {
    context: GlContext,
    canvas: Canvas<OpenGl>,
    state: State,
    event_manager: EventManager,
}

impl WindowHandler for OpenWindowExample {
    fn on_frame(&mut self) {

        self.context.make_current();

        let width = self.state.transform.get_width(self.state.root);
        let height = self.state.transform.get_height(self.state.root);

        self.canvas.set_size(width as u32, height as u32, 1.0);
        self.canvas.clear_rect(0, 0, width as u32, height as u32, Color::rgb(80, 80, 80));

        let hierarchy = self.state.hierarchy.clone();

        if self.state.apply_animations() {
            self.state.insert_event(Event::new(WindowEvent::Relayout).target(Entity::null()).origin(Entity::new(0, 0)));
            //self.state.insert_event(Event::new(WindowEvent::Redraw));
        }

        self.state.insert_event(Event::new(WindowEvent::Restyle).target(Entity::null()).origin(Entity::new(0, 0)));
        self.state.insert_event(Event::new(WindowEvent::Relayout).target(Entity::null()).origin(Entity::new(0, 0)));

        //while !self.state.event_queue.is_empty() {
            self.event_manager.flush_events(&mut self.state);



            self.event_manager.draw(&mut self.state, &hierarchy, &mut self.canvas);
            
        //}

        

        // unsafe {
        //     gl::ClearColor(1.0, 0.0, 1.0, 1.0);
        //     gl::Clear(gl::COLOR_BUFFER_BIT);
        // }





        //draw_colorwheel(&mut self.canvas, 200.0, 200.0, 200.0, 200.0, 0.0);

        self.canvas.flush();
        self.context.swap_buffers();
    }

    fn on_event(&mut self, _window: &mut baseview::Window, event: baseview::Event) {
        match event {
            baseview::Event::Mouse(e) => {
                match e {
                    baseview::MouseEvent::CursorMoved{position} => {

                        //println!("Cursor Moved");

                        self.state.insert_event(Event::new(WindowEvent::Restyle).target(Entity::null()).origin(Entity::new(0, 0)));
                        self.state.insert_event(Event::new(WindowEvent::Relayout).target(Entity::null()).origin(Entity::new(0, 0)));

                        let cursorx = (position.x) as f32;
                        let cursory = (position.y) as f32;

                        self.state.mouse.cursorx = cursorx as f32;
                        self.state.mouse.cursory = cursory as f32;

                        let mut hovered_widget = Entity::new(0, 0);

                        // This only really needs to be computed when the hierarchy changes
                        // Can be optimised
                        let mut draw_hierarchy: Vec<Entity> = self.state.hierarchy.into_iter().collect();

                        draw_hierarchy.sort_by_cached_key(|entity| self.state.transform.get_z_order(*entity));


                        for widget in draw_hierarchy.into_iter() {
                            // Skip invisible widgets
                            if self.state.transform.get_visibility(widget) == Visibility::Invisible
                            {
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

                            let border_width = match self.state
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

                            if cursorx >= posx && cursorx >= clip_posx
                                && cursorx < (posx + width) && cursorx < (clip_posx + clip_width)
                                && cursory >= posy && cursory >= clip_posy
                                && cursory < (posy + height) && cursory < (clip_posy + clip_height)
                            {
                                hovered_widget = widget;
                                if let Some(pseudo_classes) = self.state.style.pseudo_classes.get_mut(hovered_widget) {
                                    pseudo_classes.set_over(true);
                                }
                            } else {
                                if let Some(pseudo_classes) = self.state.style.pseudo_classes.get_mut(hovered_widget) {
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

                            if let Some(pseudo_classes) = self.state.style.pseudo_classes.get_mut(hovered_widget) {
                                pseudo_classes.set_hover(true);
                            }

                            if let Some(pseudo_classes) = self.state.style.pseudo_classes.get_mut(self.state.hovered) {
                                pseudo_classes.set_hover(false);
                            }

                            self.state.insert_event(Event::new(WindowEvent::MouseOver).target(hovered_widget));
                            self.state.insert_event(Event::new(WindowEvent::MouseOut).target(self.state.hovered));

                            self.state.hovered = hovered_widget;
                            self.state.active = Entity::null();

                            self.state
                                .insert_event(Event::new(WindowEvent::Restyle));
                            self.state
                                .insert_event(Event::new(WindowEvent::Redraw));
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
                    }

                    baseview::MouseEvent::ButtonPressed(button) => {
                        let b = match button {
                            baseview::MouseButton::Left => MouseButton::Left,
                            baseview::MouseButton::Right => MouseButton::Right,
                            baseview::MouseButton::Middle => MouseButton::Middle,
                            baseview::MouseButton::Other(id) => MouseButton::Other(id as u16),
                            _=> MouseButton::Left,
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
                        }

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
                                Event::new(WindowEvent::MouseDown(b))
                                    .target(self.state.hovered),
                            );
                        }

                        match b {
                            MouseButton::Left => {
                                self.state.mouse.left.pos_down = (self.state.mouse.cursorx, self.state.mouse.cursory);
                                self.state.mouse.left.pressed = self.state.hovered;
                            }

                            MouseButton::Middle => {
                                self.state.mouse.middle.pos_down = (self.state.mouse.cursorx, self.state.mouse.cursory);
                                self.state.mouse.left.pressed = self.state.hovered;
                            }
                            
                            MouseButton::Right => {
                                self.state.mouse.right.pos_down = (self.state.mouse.cursorx, self.state.mouse.cursory);
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
                            _=> MouseButton::Left,
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
                        }

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
                                Event::new(WindowEvent::MouseUp(b))
                                    .target(self.state.hovered),
                            );
                        }

                        match b {
                            MouseButton::Left => {
                                self.state.mouse.left.pos_up = (self.state.mouse.cursorx, self.state.mouse.cursory);
                                self.state.mouse.left.released = self.state.hovered;
                            }
                            
                            MouseButton::Middle => {
                                self.state.mouse.middle.pos_up = (self.state.mouse.cursorx, self.state.mouse.cursory);
                                self.state.mouse.left.released = self.state.hovered;
                            } 
                                
                            MouseButton::Right => {
                                self.state.mouse.right.pos_up = (self.state.mouse.cursorx, self.state.mouse.cursory);
                                self.state.mouse.left.released = self.state.hovered;
                            }

                            _ => {}
                        }
                    }

                    _=> {}
                }
                //println!("Mouse event: {:?}", e)
            },

            _=> {}

            /*
            baseview::Event::Keyboard(e) => {
                match e {
                    KeyboardEvent {state: s, key, code, location, modifiers, repeat, is_composing} => {
                        
                        
                        if s == KeyState::Down {
                            self.state.insert_event(
                                Event::new(WindowEvent::KeyInput(
                                    KeyboardInput {
                                        scancode: 0,
                                        virtual_keycode: Some(crate::VirtualKeyCode::Z),
                                        state: MouseButtonState::Pressed,
                                    }
                                ))
                                .target(self.state.hovered)
                            );
                        }

                        if s == KeyState::Up {
                            self.state.insert_event(
                                Event::new(WindowEvent::KeyInput(
                                    KeyboardInput {
                                        scancode: 0,
                                        virtual_keycode: Some(crate::VirtualKeyCode::Z),
                                        state: MouseButtonState::Released,
                                    }
                                ))
                                .target(self.state.hovered)
                            );
                        }


                        
                        
                    }
                }
                //println!("Keyboard event: {:?}", e);
            },
            */
            
            
            
            baseview::Event::Window(e) => println!("Window event: {:?}", e),
        }
    }
}

pub struct Application {
    //pub state: State,
    //pub event_manager: EventManager,
    //pub app_runner: Option<baseview::AppRunner>
}

impl Application {
    pub fn new<F: FnMut(WindowDescription, &mut State, Entity) -> WindowDescription>(
        mut app: F,
    ) -> Self 
    {
        
        let mut state = State::new();
        let root = state.root;

        

        WindowWidget::new().build_window(&mut state);

        state.hierarchy.add(state.root, None);

        let root = state.root;

        state.insert_event(Event::new(WindowEvent::Restyle));
        state.insert_event(Event::new(WindowEvent::Relayout).target(Entity::null()));

        let regular_font = include_bytes!("../resources/Roboto-Regular.ttf");
        let bold_font = include_bytes!("../resources/Roboto-Bold.ttf");
        let icon_font = include_bytes!("../resources/entypo.ttf");



        //let event_manager = EventManager::new();

        //let window_description = win(WindowDescription::new());

        let window_description = app(WindowDescription::new(), &mut state, root);

        let width = window_description.inner_size.to_physical(1.0).width;
        let height = window_description.inner_size.to_physical(1.0).height; 

        state.style.width.insert(
            state.root,
            Length::Pixels(width as f32),
        );

        state.style.height.insert(
            state.root,
            Length::Pixels(height as f32),
        );

        state.transform.set_width(
            state.get_root(),
            width as f32,
        );
        state.transform.set_height(
            state.get_root(),
            height as f32,
        );
        state.transform.set_opacity(state.get_root(), 1.0);

        let window_open_options = baseview::WindowOpenOptions {
            title: window_description.title,
            size: baseview::Size::new(width, height),
            scale: WindowScalePolicy::SystemScaleFactor,
            //parent: baseview::Parent::None,
        };

        baseview::Window::open_blocking(
            window_open_options,
            move |window| {
                let context = GlContext::create(window, Default::default()).unwrap();
                context.make_current();
                gl::load_with(|symbol| context.get_proc_address(symbol) as *const _);
                let renderer = OpenGl::new(|symbol| context.get_proc_address(symbol) as *const _).expect("Cannot create renderer");
                let mut canvas = Canvas::new(renderer).expect("Cannot create canvas");
    
                // let fonts = Fonts {
                //     regular: Some(canvas
                //         .add_font("examples/resources/Roboto-Regular.ttf")
                //         .expect("Cannot add font")),
                //     bold: Some(canvas
                //         .add_font("examples/resources/Roboto-Bold.ttf")
                //         .expect("Cannot add font")),
                //     icons: Some(canvas.add_font("examples/resources/entypo.ttf").expect("Cannot add font")),
                // };

                // state.fonts = fonts;

                //let mut state = state.clone();

                //win(&mut state, root);

                let fonts = Fonts {
                    regular: Some(canvas
                        .add_font_mem(regular_font)
                        .expect("Cannot add font")),
                    bold: Some(canvas
                        .add_font_mem(bold_font)
                        .expect("Cannot add font")),
                    icons: Some(canvas.add_font_mem(icon_font).expect("Cannot add font")),
                };
        
                state.fonts = fonts;

                OpenWindowExample {context, canvas, state, event_manager: EventManager::new()}
            } 
        );

 
        Application {
            // event_manager: event_manager,
            // state: state,
            //app_runner: opt_app_runner,
        }
    }


    pub fn new_with_parent<P, F: FnMut(WindowDescription, &mut State, Entity) -> WindowDescription>(
        parent: &P, mut app: F,
    ) -> Self 
    
    where P: HasRawWindowHandle
    
    {
        
        let mut state = State::new();
        let root = state.root;

        

        WindowWidget::new().build_window(&mut state);

        state.hierarchy.add(state.root, None);

        let root = state.root;

        state.insert_event(Event::new(WindowEvent::Restyle));
        state.insert_event(Event::new(WindowEvent::Relayout).target(Entity::null()));

        let regular_font = include_bytes!("../resources/Roboto-Regular.ttf");
        let bold_font = include_bytes!("../resources/Roboto-Bold.ttf");
        let icon_font = include_bytes!("../resources/entypo.ttf");



        //let event_manager = EventManager::new();

        //let window_description = win(WindowDescription::new());

        let window_description = app(WindowDescription::new(), &mut state, root);

        let width = window_description.inner_size.to_physical(1.0).width;
        let height = window_description.inner_size.to_physical(1.0).height; 

        state.style.width.insert(
            state.root,
            Length::Pixels(width as f32),
        );

        state.style.height.insert(
            state.root,
            Length::Pixels(height as f32),
        );

        state.transform.set_width(
            state.get_root(),
            width as f32,
        );
        state.transform.set_height(
            state.get_root(),
            height as f32,
        );
        state.transform.set_opacity(state.get_root(), 1.0);

        let window_open_options = baseview::WindowOpenOptions {
            title: window_description.title,
            size: baseview::Size::new(width, height),
            scale: WindowScalePolicy::SystemScaleFactor,
            //parent: baseview::Parent::None,
        };

        baseview::Window::open_parented(
            parent,
            window_open_options,
            move |window| {
                let context = GlContext::create(window, Default::default()).unwrap();
                context.make_current();
                gl::load_with(|symbol| context.get_proc_address(symbol) as *const _);
                let renderer = OpenGl::new(|symbol| context.get_proc_address(symbol) as *const _).expect("Cannot create renderer");
                let mut canvas = Canvas::new(renderer).expect("Cannot create canvas");
    
                // let fonts = Fonts {
                //     regular: Some(canvas
                //         .add_font("examples/resources/Roboto-Regular.ttf")
                //         .expect("Cannot add font")),
                //     bold: Some(canvas
                //         .add_font("examples/resources/Roboto-Bold.ttf")
                //         .expect("Cannot add font")),
                //     icons: Some(canvas.add_font("examples/resources/entypo.ttf").expect("Cannot add font")),
                // };

                // state.fonts = fonts;

                //let mut state = state.clone();

                //win(&mut state, root);

                let fonts = Fonts {
                    regular: Some(canvas
                        .add_font_mem(regular_font)
                        .expect("Cannot add font")),
                    bold: Some(canvas
                        .add_font_mem(bold_font)
                        .expect("Cannot add font")),
                    icons: Some(canvas.add_font_mem(icon_font).expect("Cannot add font")),
                };
        
                state.fonts = fonts;

                OpenWindowExample {context, canvas, state, event_manager: EventManager::new()}
            } 
        );

 
        Application {
            // event_manager: event_manager,
            // state: state,
            //app_runner: opt_app_runner,
        }
    }

}
