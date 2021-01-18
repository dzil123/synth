use crate::util::{lerp, BITRATE_F};

// all units seconds except percent
#[derive(Clone, PartialEq)]
pub struct ADSRParams {
    pub attack_length: f32,
    pub decay_length: f32,
    pub sustain_percent: f32,
    pub sustain_length: f32,
    pub release_length: f32,
    pub quiet_length: f32,
}

impl ADSRParams {
    fn assert(&self) {
        assert!(self.attack_length >= 0.0);
        assert!(self.decay_length >= 0.0);
        assert!(0.0 <= self.sustain_percent && self.sustain_percent <= 1.0);
        assert!(self.sustain_length >= 0.0);
        assert!(self.release_length >= 0.0);
        assert!(self.quiet_length >= 0.0);
    }

    pub fn build(self) -> ADSR {
        self.assert();

        ADSR {
            params: self,
            ..Default::default()
        }
    }

    pub fn zero() -> Self {
        Self {
            attack_length: 0.0,
            decay_length: 0.0,
            sustain_percent: 0.0,
            sustain_length: 0.0,
            release_length: 0.0,
            quiet_length: 0.0,
        }
    }

    pub fn flat(length: f32) -> Self {
        Self {
            sustain_percent: 1.0,
            sustain_length: length,
            ..Self::zero()
        }
    }

    pub fn flat2(length: f32, quiet: f32) -> Self {
        Self {
            quiet_length: quiet,
            ..Self::flat(length)
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
            quiet_length: 0.5,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum State {
    Attack,
    Decay,
    Sustain,
    Release,
    Quiet,
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

    // if true, the ending of this envelope can be cut short (interrupted)
    pub fn is_done(&self) -> bool {
        use State::*;
        match self.state {
            Release | Quiet | End => true,
            _ => false,
        }
    }

    pub fn release(&mut self) {
        // allow to be released multiple times, with subsequent releases ignored
        if !self.is_done() {
            self.switch_state(State::Release);
        }
    }

    pub fn next(&mut self) -> Option<f32> {
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
                    self.switch_state(State::Quiet);
                    0.0
                } else {
                    lerp(
                        self.progress as f32 / duration_f,
                        self.params.sustain_percent,
                        0.0,
                    )
                }
            }
            State::Quiet => {
                let duration_f = self.params.quiet_length * BITRATE_F;
                if self.progress >= (duration_f as u32) {
                    self.switch_state(State::End);
                }
                0.0
            }
            // State::End => 0.0,
            State::End => return None,
        };

        self.progress += 1;

        Some(x)
    }
}
