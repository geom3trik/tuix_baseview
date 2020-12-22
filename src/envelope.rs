

pub struct EnvelopeASDR {
    pub attack_time: f32,
    pub decay_time: f32,
    pub sustain_amp: f32,
    pub release_time: f32,

    pub start_amplitude: f32,

    pub trigger_on_time: f32,
    pub trigger_off_time: f32,

    pub note_on: bool,


}

impl EnvelopeASDR {
    pub fn new() -> Self {
        EnvelopeASDR {
            attack_time: 0.1,
            decay_time: 0.1,
            sustain_amp: 0.5,
            release_time: 0.1,
            start_amplitude: 0.5,
            trigger_on_time: 0.0,
            trigger_off_time: 0.0,
            note_on: false,
        }
    }

    pub fn get_amplitude(&self, time: f32) -> f32 {
        let mut amplitude = 0.0;
        let mut release_amplitude = 0.0;

        
        if self.trigger_on_time > self.trigger_off_time { // Note is on
            let lifetime = time - self.trigger_on_time;
            
            if lifetime <= self.attack_time {
                amplitude = (lifetime / self.attack_time) * self.start_amplitude;
            }

            if lifetime > self.attack_time && lifetime <= (self.attack_time + self.decay_time) {
                amplitude = ((lifetime - self.attack_time)/self.decay_time) * (self.sustain_amp - self.start_amplitude) + self.start_amplitude;
            }
             
            if lifetime > (self.attack_time + self.decay_time) {
                amplitude = self.sustain_amp;
            }

        } else { // Note is off
            let lifetime = self.trigger_off_time - self.trigger_on_time;

            if lifetime <= self.attack_time {
                release_amplitude = (lifetime / self.attack_time) * self.start_amplitude;
            }

            if lifetime > self.attack_time && lifetime <= (self.attack_time + self.decay_time) {
                release_amplitude = ((lifetime - self.attack_time)/self.decay_time) * (self.sustain_amp - self.start_amplitude) + self.start_amplitude;
            }

            if lifetime > (self.attack_time + self.decay_time) {
                release_amplitude = self.sustain_amp;
            }

            amplitude = ((time - self.trigger_off_time) / self.release_time) * (0.0 - release_amplitude) + release_amplitude;

        }

        // if self.note_on {

        //     // amplitude = 1.0;
        //     //ADS
        //     //Attack
        //     if lifetime < self.attack_time {
        //         amplitude = (lifetime / self.attack_time) * self.start_amplitude;
        //     } else if lifetime > self.attack_time && lifetime < (self.attack_time + self.decay_time) {
        //         amplitude = ((lifetime - self.attack_time)/self.decay_time) * (self.sustain_amp - self.start_amplitude) + self.start_amplitude;
        //     } else if lifetime > (self.attack_time + self.decay_time) {
        //         amplitude = self.sustain_amp;
        //     }


        // } else {
        //     // R
        //     amplitude = ((time - self.trigger_off_time) / self.release_time) * (0.0 - self.sustain_amp) + self.sustain_amp;
        // }

        if amplitude <= 0.01 {
            amplitude = 0.0;
        }

        return amplitude;

    }

    pub fn note_on(&mut self, time_on: f32) {
        // println!("ON: {}", time_on);
        if !self.note_on {
            self.trigger_on_time = time_on;
            self.note_on = true;            
        }

    }

    pub fn note_off(&mut self, time_off: f32) {
        // println!("OFF: {}", time_off);
        self.trigger_off_time = time_off;
        self.note_on = false;
    }

}