
extern crate anyhow;
extern crate cpal;

use::tuix::*;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

use std::thread;
use std::time::Duration;
use std::sync::mpsc::channel;

pub mod envelope;
pub use envelope::EnvelopeASDR;

use femtovg::{
    renderer::OpenGl,
    Baseline,
    Canvas,
    FillRule,
    FontId,
    ImageFlags,
    ImageId,
    LineCap,
    LineJoin,
    Paint,
    Path,
    Renderer,
    Solidity,
};

static THEME: &'static str = include_str!("theme.css");


#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Message {
    Note(f32),
    Key(bool),
    CarrierFrequency(f32),
    ModulationFrequency(f32),
    ModulationAmplitude(f32),
    Attack(f32),
    Sustain(f32),
    Decay(f32),
    Release(f32),
}

pub fn oscillator(osc_type: u8, time: f32, phi1: f32, phi2: f32, ma: f32) -> f32 {
    match osc_type {
        0 => {  // Sine
            // println!("{}", mf);
            return (2.0 * 3.141592 * phi1 + ma*(2.0 * 3.141592 * phi2).cos()).sin();
        }

        1 => { // Square
            return if (2.0*3.141592*phi1).sin() > 0.0 {
                0.1
            } else {
                -0.1
            }
        }

        2 => { // Triangle
            return  0.1*((2.0*3.141592*phi1).sin() * 2.0 / 3.141592).asin();
        }

        // 3 => { // Saw
        //     return 0.1*(2.0*3.141592) * (phi1*3.141592*(time % (1.0 /phi1)) - (3.141592/2.0));
        // }

        _=> {
            return 0.0;
        }
    }
}


pub struct TestWidget {
    command_sender: crossbeam_channel::Sender<Message>,
    freq_knob: Entity,
    modf_knob: Entity,
    moda_knob: Entity,
}

impl TestWidget {
    pub fn new(command_sender: crossbeam_channel::Sender<Message>) -> Self {
        TestWidget {
            command_sender,
            freq_knob: Entity::null(),
            modf_knob: Entity::null(),
            moda_knob: Entity::null(),
        }
    }
}

impl BuildHandler for TestWidget {
    type Ret = Entity;
    fn on_build(&mut self, state: &mut State, entity: Entity) -> Self::Ret {

        state.focused = entity;

        let rvbox = ResizableVBox::new().build(state, entity, |builder| 
            builder
                .set_width(Length::Pixels(300.0))
                .set_height(Length::Percentage(1.0))
                .set_background_color(Color::rgb(100,50,50))
        );

        let panel = Panel::new("Oscillator").build(state, rvbox, |builder| builder);
    

        let row = HBox::new().build(state, panel, |builder| {
            builder.set_justify_content(JustifyContent::SpaceEvenly).set_margin_bottom(Length::Pixels(5.0))
        });

        self.freq_knob = ValueKnob::new("Freq", 440.0, 0.0, 2000.0).build(state, row, |builder|
            builder
                .set_width(Length::Pixels(50.0))
        );

        let panel = Panel::new("Modulation").build(state, rvbox, |builder| builder);

        let row = HBox::new().build(state, panel, |builder| {
            builder.set_justify_content(JustifyContent::SpaceEvenly).set_margin_bottom(Length::Pixels(5.0))
        });

        self.modf_knob = ValueKnob::new("Frequency", 0.0, 0.0, 100.0).build(state, row, |builder|
            builder
                .set_width(Length::Pixels(50.0))
        );

        self.moda_knob = ValueKnob::new("Amplitude", 1.0, 0.0, 1.0).build(state, row, |builder|
            builder
                .set_width(Length::Pixels(50.0))
        );

        
        let panel = Panel::new("Envelope").build(state, rvbox, |builder| builder);

        let envelope_viewer = EnvelopeViewer::new(self.command_sender.clone()).build(state, panel, |builder| builder);
    

        entity.set_width(state, Length::Percentage(1.0)).set_height(state, Length::Percentage(1.0))

    }
}

impl EventHandler for TestWidget {
    fn on_event(&mut self, state: &mut State, entity: Entity, event: &mut Event) -> bool {

        if let Some(window_event) = event.is_type::<WindowEvent>() {
            match window_event {
                WindowEvent::MouseDown(button) => {
                    //println!("Mouse Down");
                    // if let Err(err) = self.command_sender.send(Message::Key(true)) {
                    //     assert_eq!(err.into_inner(), Message::Key(true));
                    // }
                }

                WindowEvent::MouseUp(button) => {
                    //println!("Mouse Up");
                    //self.command_sender.send(false).unwrap();
                    // if let Err(err) = self.command_sender.send(Message::Key(false)) {
                    //     assert_eq!(err.into_inner(), Message::Key(false));
                    // }
                }

                WindowEvent::KeyInput(key_input) => {
                    if key_input.state == MouseButtonState::Pressed {
                        //println!("Pressed Key");
                        if let Err(err) = self.command_sender.send(Message::Key(true)) {
                            assert_eq!(err.into_inner(), Message::Key(true));
                        }

                        match key_input.virtual_keycode {
                            Some(VirtualKeyCode::Z) => {
                                println!("Z");
                                if let Err(err) = self.command_sender.send(Message::Note(110.0)) {
                                    assert_eq!(err.into_inner(), Message::Note(110.0));
                                }
                            }

                            Some(VirtualKeyCode::S) => {
                                //println!("S");
                                if let Err(err) = self.command_sender.send(Message::Note(116.54)) {
                                    assert_eq!(err.into_inner(), Message::Note(116.54));
                                }
                            }

                            Some(VirtualKeyCode::X) => {
                                //println!("X");
                                if let Err(err) = self.command_sender.send(Message::Note(123.47)) {
                                    assert_eq!(err.into_inner(), Message::Note(123.47));
                                }
                            }

                            Some(VirtualKeyCode::D) => {
                                //println!("X");
                                if let Err(err) = self.command_sender.send(Message::Note(130.81)) {
                                    assert_eq!(err.into_inner(), Message::Note(130.81));
                                }
                            }

                            Some(VirtualKeyCode::C) => {
                                //println!("X");
                                if let Err(err) = self.command_sender.send(Message::Note(138.59)) {
                                    assert_eq!(err.into_inner(), Message::Note(138.59));
                                }
                            }

                            Some(VirtualKeyCode::F) => {
                                //println!("X");
                                if let Err(err) = self.command_sender.send(Message::Note(146.83)) {
                                    assert_eq!(err.into_inner(), Message::Note(146.83));
                                }
                            }

                            Some(VirtualKeyCode::V) => {
                                //println!("X");
                                if let Err(err) = self.command_sender.send(Message::Note(155.56)) {
                                    assert_eq!(err.into_inner(), Message::Note(155.56));
                                }
                            }

                            Some(VirtualKeyCode::G) => {
                                //println!("X");
                                if let Err(err) = self.command_sender.send(Message::Note(164.81)) {
                                    assert_eq!(err.into_inner(), Message::Note(164.81));
                                }
                            }

                            Some(VirtualKeyCode::B) => {
                                //println!("X");
                                if let Err(err) = self.command_sender.send(Message::Note(174.61)) {
                                    assert_eq!(err.into_inner(), Message::Note(174.61));
                                }
                            }

                            Some(VirtualKeyCode::H) => {
                                //println!("X");
                                if let Err(err) = self.command_sender.send(Message::Note(185.0)) {
                                    assert_eq!(err.into_inner(), Message::Note(185.0));
                                }
                            }

                            Some(VirtualKeyCode::N) => {
                                //println!("X");
                                if let Err(err) = self.command_sender.send(Message::Note(196.0)) {
                                    assert_eq!(err.into_inner(), Message::Note(196.0));
                                }
                            }

                            Some(VirtualKeyCode::J) => {
                                //println!("X");
                                if let Err(err) = self.command_sender.send(Message::Note(207.65)) {
                                    assert_eq!(err.into_inner(), Message::Note(207.65));
                                }
                            }

                            Some(VirtualKeyCode::M) => {
                                //println!("X");
                                if let Err(err) = self.command_sender.send(Message::Note(220.0)) {
                                    assert_eq!(err.into_inner(), Message::Note(220.0));
                                }
                            }

                            _=> {}
                        }
                    } else {
                        //println!("Released Key");
                        if let Err(err) = self.command_sender.send(Message::Key(false)) {
                            assert_eq!(err.into_inner(), Message::Key(false));
                        }
                    }
                }

                _=> {}
            }
        }

        if let Some(slider_event) = event.message.downcast::<SliderEvent>() {
            match slider_event {
                
                SliderEvent::ValueChanged(_, val) => {
                    //println!("Val: {} {}", event.target, val);
                    if event.target == self.freq_knob {
                        if let Err(err) = self.command_sender.send(Message::CarrierFrequency(*val)) {
                            assert_eq!(err.into_inner(), Message::CarrierFrequency(*val));
                        }
                    }

                    if event.target == self.modf_knob {
                        if let Err(err) = self.command_sender.send(Message::ModulationFrequency(*val)) {
                            assert_eq!(err.into_inner(), Message::ModulationFrequency(*val));
                        }
                    }

                    if event.target == self.moda_knob {
                        if let Err(err) = self.command_sender.send(Message::ModulationAmplitude(*val)) {
                            assert_eq!(err.into_inner(), Message::ModulationAmplitude(*val));
                        }
                    }
                    
                }

                _=> {}
            }
        }

        return false;
    }
}

pub struct EnvelopeViewer {

    command_sender: crossbeam_channel::Sender<Message>,

    scope: Entity,
    knobs_container: Entity,
    attack_knob: Entity,
    decay_knob: Entity,
    sustain_knob: Entity,
    release_knob: Entity,

    attack: f32,
    decay: f32,
    sustain: f32,
    release: f32,

}

impl EnvelopeViewer {
    pub fn new(command_sender: crossbeam_channel::Sender<Message>) -> Self {
        EnvelopeViewer {
            command_sender,
            scope: Entity::null(),
            knobs_container: Entity::null(),
            attack_knob: Entity::null(),
            decay_knob: Entity::null(),
            sustain_knob: Entity::null(),
            release_knob: Entity::null(),

            attack: 0.1,
            decay: 0.1,
            sustain: 0.5,
            release: 0.1,
        }
    }
}

impl BuildHandler for EnvelopeViewer {
    type Ret = Entity;
    fn on_build(&mut self, state: &mut State, entity: Entity) -> Self::Ret {

        entity.set_flex_direction(state, FlexDirection::Column);


        self.scope = Button::new().build(state, entity, |builder|
            builder
                //.set_width(Length::Pixels(100.0))
                .set_height(Length::Pixels(70.0))
                .set_background_color(Color::rgb(50,150,50))
                .set_margin_bottom(Length::Pixels(5.0))
                .set_visibility(Visibility::Invisible)
        );

        let row = HBox::new().build(state, entity, |builder| {
            builder.set_justify_content(JustifyContent::SpaceEvenly).set_margin_bottom(Length::Pixels(5.0))
        });

        self.attack_knob = ValueKnob::new("Attack", 0.1, 0.0, 1.0).build(state, row, |builder|
            builder
                .set_width(Length::Pixels(50.0))
        );

        self.decay_knob = ValueKnob::new("Decay", 0.1, 0.001, 1.0).build(state, row, |builder|
            builder
                .set_width(Length::Pixels(50.0))
        );

        self.sustain_knob = ValueKnob::new("Sustain", 0.5, 0.0, 1.0).build(state, row, |builder|
            builder
                .set_width(Length::Pixels(50.0))
        );

        self.release_knob = ValueKnob::new("Release", 0.1, 0.01, 1.0).build(state, row, |builder|
            builder
                .set_width(Length::Pixels(50.0))
        );

        entity
    }
} 

impl EventHandler for EnvelopeViewer {
    fn on_event(&mut self, state: &mut State, entity: Entity, event: &mut Event) -> bool {
        if let Some(slider_event) = event.message.downcast::<SliderEvent>() {
            match slider_event {
                
                SliderEvent::ValueChanged(_, val) => {
                    //println!("Val: {} {}", event.target, val);
                    if event.target == self.attack_knob {
                        println!("Attack: {} {}", event.target, val);
                        self.attack = *val;
                        if let Err(err) = self.command_sender.send(Message::Attack(*val)) {
                            assert_eq!(err.into_inner(), Message::Attack(*val));
                        }
                    }

                    if event.target == self.decay_knob {
                        println!("Decay: {} {}", event.target, val);
                        self.decay = *val;
                        if let Err(err) = self.command_sender.send(Message::Decay(*val)) {
                            assert_eq!(err.into_inner(), Message::Decay(*val));
                        }
                    }

                    if event.target == self.sustain_knob {
                        println!("Sustain: {} {}", event.target, val);
                        self.sustain = *val;
                        if let Err(err) = self.command_sender.send(Message::Sustain(*val)) {
                            assert_eq!(err.into_inner(), Message::Sustain(*val));
                        }
                    }

                    if event.target == self.release_knob {
                        println!("Release: {} {}", event.target, val);
                        self.release = *val;
                        if let Err(err) = self.command_sender.send(Message::Release(*val)) {
                            assert_eq!(err.into_inner(), Message::Release(*val));
                        }
                    }
                    
                }

                _=> {}
            }
        }

        false
    }

    fn on_draw(&mut self, state: &mut State, entity: Entity, canvas: &mut Canvas<OpenGl>) {
        
        
        let opacity = state.transform.get_opacity(self.scope);

        let mut background_color: femtovg::Color = state.style.background_color.get(self.scope).cloned().unwrap_or_default().into();
        background_color.set_alphaf(background_color.a * opacity);

        let posx = state.transform.get_posx(self.scope);
        let posy = state.transform.get_posy(self.scope);
        let width = state.transform.get_width(self.scope);
        let height = state.transform.get_height(self.scope);

        //println!("posx: {} posy: {} width: {} height: {}", posx, posy, width, height);

        let cx = posx + 0.5 * width;
        let cy = posy + 0.5 * height;

        let r1 = height/2.0;
        let r0 = r1 - 10.0;

        let division = (width - 14.0) / 4.0;
        //let division = 30.0;

        let op = (opacity * 256.0) as u8;

        

        let attack_pos =  division * self.attack;
        let decay_pos = attack_pos + (division * self.decay);
        let release_pos = decay_pos + division + (division * self.release);

        let sustain_pos = (height - 14.0) * (1.0 - self.sustain);

        canvas.save();

        let mut path = Path::new();
        path.rounded_rect(posx, posy, width, height, 3.0);
        canvas.fill_path(&mut path, Paint::color(femtovg::Color::rgba(46, 46, 46, op)));

        let mut path = Path::new();
        path.move_to(posx, posy + height - 7.0);
        path.line_to(posx + width, posy + height - 7.0);
        canvas.fill_path(&mut path, Paint::color(femtovg::Color::rgba(66, 66, 66, op)));

        let mut path = Path::new();
        path.move_to(posx + 7.0, posy + height - 7.0);
        path.line_to(posx + attack_pos + 7.0, posy + 7.0);
        path.line_to(posx + decay_pos + 7.0, posy + sustain_pos + 7.0);
        path.line_to(posx + decay_pos + 7.0 + division, posy + sustain_pos + 7.0);
        path.line_to(posx + release_pos + 7.0, posy + height - 7.0);
        let mut paint = Paint::color(femtovg::Color::rgba(54, 105, 201, op));
        paint.set_line_width(2.0);
        paint.set_line_join(LineJoin::Bevel);
        canvas.stroke_path(&mut path, paint);

        let mut path = Path::new();
        path.circle(posx + 7.0, posy + height - 7.0, 4.0);
        canvas.fill_path(&mut path, Paint::color(femtovg::Color::rgba(54, 105, 201, op)));

        let mut path = Path::new();
        path.circle(posx + attack_pos + 7.0, posy + 7.0, 4.0);
        canvas.fill_path(&mut path, Paint::color(femtovg::Color::rgba(54, 105, 201, op)));

        let mut path = Path::new();
        path.circle(posx + decay_pos + 7.0, posy + sustain_pos + 7.0, 4.0);
        canvas.fill_path(&mut path, Paint::color(femtovg::Color::rgba(54, 105, 201, op)));

        let mut path = Path::new();
        path.circle(posx + decay_pos + 7.0 + division, posy + sustain_pos + 7.0, 4.0);
        canvas.fill_path(&mut path, Paint::color(femtovg::Color::rgba(54, 105, 201, op)));

        let mut path = Path::new();
        path.circle(posx + release_pos + 7.0, posy + height - 7.0, 4.0);
        canvas.fill_path(&mut path, Paint::color(femtovg::Color::rgba(54, 105, 201, op)));

        // let mut path = Path::new();
        // path.circle(cx, cy, r1);
        // let mut paint = Paint::color(background_color);
        // canvas.fill_path(&mut path, paint);

        canvas.restore();


    }
}



pub struct ResizableVBox {
    resizing: bool,
    previous_width: f32,
}

impl ResizableVBox {
    pub fn new() -> Self {
        ResizableVBox {
            resizing: false,
            previous_width: 0.0,
        }
    }
}

impl BuildHandler for ResizableVBox {
    type Ret = Entity;
    fn on_build(&mut self, state: &mut State, entity: Entity) -> Self::Ret {
        entity
            .set_width(state, Length::Pixels(300.0))
            .set_max_width(state, Length::Pixels(500.0))
            .set_min_width(state, Length::Pixels(300.0));
        //state.style.z_order.set(self.resize_marker, 1);

        entity
    }
}

impl EventHandler for ResizableVBox {
    fn on_event(&mut self, state: &mut State, entity: Entity, event: &mut Event) -> bool {
        
        if let Some(window_event) = event.is_type::<WindowEvent>() {
            match window_event {
                WindowEvent::MouseDown(button) => {
                    if *button == MouseButton::Left {
                        if state.mouse.left.pos_down.0 >= state.transform.get_posx(entity) + state.transform.get_width(entity) - 4.0
                            && state.mouse.left.pos_down.0 <= state.transform.get_posx(entity) + state.transform.get_width(entity)
                        {
                            self.resizing = true;
                            self.previous_width = state.transform.get_width(entity);
                            state.capture(entity);
                        }
                    }
                }

                WindowEvent::MouseUp(button) => {
                    if *button == MouseButton::Left {
                        if self.resizing == true {
                            //state.release(entity);
                            self.resizing = false;
                            state.insert_event(Event::new(WindowEvent::MouseMove(state.mouse.cursorx, state.mouse.cursory)).target(entity));
                        }
                    }
                }

                // Occurs when the cursor leaves the entity
                WindowEvent::MouseOut => {
                    if !self.resizing {
                        state.insert_event(Event::new(WindowEvent::SetCursor(CursorIcon::Arrow)));
                    }
                    
                }

                WindowEvent::MouseMove(x, y) => {
                
                    if self.resizing {
                        let distx =  *x - state.mouse.left.pos_down.0;
                        entity.set_width(state, Length::Pixels(self.previous_width + distx));
                    } else {
                        if *x > state.transform.get_posx(entity) + state.transform.get_width(entity) - 4.0
                            && *x < state.transform.get_posx(entity) + state.transform.get_width(entity)
                        {
                            state.insert_event(Event::new(WindowEvent::SetCursor(CursorIcon::EResize)));
                            
                        } else {

                            state.insert_event(Event::new(WindowEvent::SetCursor(CursorIcon::Arrow)));
                            state.release(entity);
                        }
                    }
                    
                }

                _ => {}
            }
        }

        false
    }
}


fn main() {



    let (command_sender, command_receiver) = crossbeam_channel::bounded(1024);


    thread::spawn(move || {
        // Manually check for flags. Can be passed through cargo with -- e.g.
        // cargo run --release --example beep --features jack -- --jack
        let host = cpal::default_host();

        println!("Device: {:?}", host.id());

        let device = host
            .default_output_device()
            .expect("failed to find a default output device");

        println!("Device: {:?}", device.name().unwrap());
        let config = device.default_output_config().unwrap();

        

        match config.sample_format() {
            cpal::SampleFormat::F32 => {
                run::<f32>(&device, &config.into(), command_receiver.clone()).unwrap();
            }

            cpal::SampleFormat::I16 => {
                run::<i16>(&device, &config.into(), command_receiver.clone()).unwrap();
            }
            
            cpal::SampleFormat::U16 => {
                run::<u16>(&device, &config.into(), command_receiver.clone()).unwrap();
            }
            
            
        }



        //std::thread::sleep(std::time::Duration::from_millis(5000));
    });


    let mut app = ApplicationBV::new(move |state, root| {

        state.style.parse_theme(THEME);

        let test = TestWidget::new(command_sender.clone()).build(state, root, |builder| builder);

        //window
    });



    app.run();


    
}

fn run<T>(device: &cpal::Device, config: &cpal::StreamConfig, command_receiver: crossbeam_channel::Receiver<Message>) -> Result<(), anyhow::Error>
where
    T: cpal::Sample,
{


    let sample_rate = config.sample_rate.0 as f32;
    let channels = config.channels as usize;

    

    // Produce a sinusoid of maximum amplitude.
    let mut base_f = 110.0;
    let mut f = 440.0;
    let mut mf = 0.0;
    let mut ma = 1.0;

    let mut envelope = EnvelopeASDR::new();

    //let octave_base_frequency = 110.0f64;
    //let twelth_root_of_2 = (2.0f64).powf(1.0/12.0);

    let mut sample_clock = 0f32;

    let mut phi1 = 0.0;
    let mut phi2 = 0.0;

    let err_fn = |err| eprintln!("an error occurred on stream: {}", err);

    let stream = device.build_output_stream(
        config,
        move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
            
            //while true {
                //let received = command_receiver.recv().unwrap();

                //let mut freq = 440.0;
                
                
                // if received {
                //     println!("Received Message: {:?}", received);
                // }

                // sample_clock = (sample_clock + 1.0) % sample_rate;
                // let sample = (sample_clock * freq * 2.0 * 3.141592 / sample_rate).sin();
                
                
                // for frame in data.chunks_mut(channels) {
                //     let value: T = cpal::Sample::from::<f32>(&sample);
                //     for sample in frame.iter_mut() {
                //         *sample = value;
                //     }
                // }

                
                //println!("Sample Clock: {}", sample_rate);
                

                for frame in data.chunks_mut(channels) {

                    

                    

                    // phi += f * 2.0 * 3.141592 / sample_rate;

                    sample_clock = (sample_clock + 1.0);
                    let time = sample_clock / sample_rate;

                    while let Ok(command) = command_receiver.try_recv() {
                       // println!("Received Message: {:?}", command);
                        match command {
                            Message::Key(state) => {
                                if state {
                                    // a = 0.1;
                                    envelope.note_on(time);
                                } else {
                                    // a = 0.0;
                                    envelope.note_off(time);
                                }
                            }
    
                            Message::Note(freq) => {
                                base_f = freq;
                                //println!("f: {}", f);
                            }

                            Message::CarrierFrequency(val) => {
                                //println!("f: {}", val);
                                f = val;
                            }

                            Message::ModulationFrequency(val) => {
                                
                                mf = val;
                                // println!("mf: {}", val);
                            }

                            Message::ModulationAmplitude(val) => {
                                
                                ma = val;
                                //println!("ma: {}", val);
                            }

                            Message::Attack(val) => {
                                envelope.attack_time = val;
                            }

                            Message::Decay(val) => {
                                envelope.decay_time = val;
                            }

                            Message::Sustain(val) => {
                                envelope.sustain_amp = val;
                            }

                            Message::Release(val) => {
                                envelope.release_time = val;
                            }
    
                            _=> {}
                        }
                        
                    }



                    phi1 = (phi1 + (f / sample_rate)).fract();
                    phi2 = (phi2 + (mf / sample_rate)).fract();

                    // if mf / sample_rate == 0.0 {
                    //     phi2 = 0.0;
                    // }

                    //println!("Phi2: {}", phi2);

                    
                    let value: T = cpal::Sample::from::<f32>(&make_noise(time, envelope.get_amplitude(time), phi1, phi2, ma));
                    for sample in frame.iter_mut() {
                        *sample = value;
                    }
                }


                //write_data(data, channels, &mut next_value)                
            //}

        },
        err_fn,
    )?;
    stream.play()?;


    //std::thread::sleep(std::time::Duration::from_millis(5000));
    std::thread::park();


    Ok(())
}

fn make_noise(time: f32, a: f32, phi1: f32, phi2: f32, ma: f32) -> f32 {

    //println!("Amp: {}", a);
    let output = a * oscillator(0,time,phi1,phi2,ma);
    //let output = a * (time * f * 2.0 * 3.141592).sin();

    // if output > 0.0 {
    //     return 0.2;
    // } else {
    //     return -0.2;
    // }

    return output;
}

fn write_data<T>(output: &mut [T], channels: usize, next_sample: &mut dyn FnMut() -> f32)
where
    T: cpal::Sample,
{
    for frame in output.chunks_mut(channels) {
        let value: T = cpal::Sample::from::<f32>(&next_sample());
        for sample in frame.iter_mut() {
            *sample = value;
        }
    }
}
