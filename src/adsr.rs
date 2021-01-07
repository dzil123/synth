use crate::util::{lerp, BITRATE_F};

// all units seconds except percent
#[derive(Clone, PartialEq)]
pub struct ADSRParams {
    pub attack_length: f32,
    pub decay_length: f32,
    pub sustain_percent: f32,
    pub sustain_length: f32,
    pub release_length: f32,
}

impl ADSRParams {
    fn assert(&self) {
        assert!(self.attack_length >= 0.0);
        assert!(self.decay_length >= 0.0);
        assert!(0.0 <= self.sustain_percent && self.sustain_percent <= 1.0);
        assert!(self.sustain_length >= 0.0);
        assert!(self.release_length >= 0.0);
    }

    pub fn build(self) -> ADSR {
        self.assert();

        ADSR {
            params: self,
            ..Default::default()
        }
    }
}

impl Default for ADSRParams {
    fn default() -> Self {
        Self {
            attack_length: 0.5,
            decay_length: 0.25,
            sustain_percent: 0.7,
            sustain_length: 1.25,
            release_length: 1.0,
        }
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
enum State {
    Attack,
    Decay,
    Sustain,
    Release,
    End,
}

impl Default for State {
    fn default() -> Self {
        Self::Attack
    }
}

#[derive(Default, Clone)]
pub struct ADSR {
    params: ADSRParams,
    state: State,
    progress: u32,
}

impl ADSR {
    pub fn copy(&self) -> Self {
        self.params.clone().build()
    }

    pub fn reset(&mut self) {
        *self = self.copy();
    }

    pub fn is_end(&self) -> bool {
        self.state == State::End
    }

    fn switch_state(&mut self, state: State) {
        self.state = state;
        self.progress = 0;
    }
}

impl Iterator for ADSR {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        let x: f32 = match &self.state {
            State::Attack => {
                let duration_f = self.params.attack_length * BITRATE_F;

                if self.progress >= (duration_f as u32) {
                    self.switch_state(State::Decay);
                    1.0
                } else {
                    self.progress as f32 / duration_f
                }
            }
            State::Decay => {
                let duration_f = self.params.decay_length * BITRATE_F;

                if self.progress >= (duration_f as u32) {
                    self.switch_state(State::Sustain);
                    self.params.sustain_percent
                } else {
                    lerp(
                        self.progress as f32 / duration_f,
                        1.0,
                        self.params.sustain_percent,
                    )
                }
            }
            State::Sustain => {
                let duration_f = self.params.sustain_length * BITRATE_F;

                if self.progress >= (duration_f as u32) {
                    self.switch_state(State::Release);
                }

                self.params.sustain_percent
            }
            State::Release => {
                let duration_f = self.params.release_length * BITRATE_F;

                if self.progress >= (duration_f as u32) {
                    self.switch_state(State::End);
                    0.0
                } else {
                    lerp(
                        self.progress as f32 / duration_f,
                        self.params.sustain_percent,
                        0.0,
                    )
                }
            }
            State::End => 0.0,
        };

        self.progress += 1;

        Some(x)
    }
}