/// Generates a pure tone sine wave
///
/// Code adapted from an example in the Embassy crate by Christian Perez, MIT or Apache 2.0 license
///
use core::f32::consts::PI;
use embassy_nrf::i2s::Sample as _;

pub type Sample = i16;
pub const NUM_SAMPLES: usize = 32;

pub struct Waveform {
    modulo: f32,
    phase_increment: f32,
}

impl Waveform {
    pub fn new(freq: f32, sample_rate: f32) -> Self {
        let modulo = 0.0;
        let phase_increment = freq / sample_rate;

        Self {
            modulo,
            phase_increment,
        }
    }

    pub fn next(&mut self, buf: &mut [Sample]) {
        for sample in buf {
            *sample = self.generate();
        }
    }

    pub fn zero(&mut self, buf: &mut [Sample]) {
        for x in buf {
            *x = 0;
        }
    }

    fn generate(&mut self) -> Sample {
        let signal = parabolic_sin(self.modulo);
        self.modulo += self.phase_increment;

        if self.modulo < 0.0 {
            self.modulo += 1.0;
        } else if self.modulo > 1.0 {
            self.modulo -= 1.0;
        }

        // if you want to attenuate the signal, do it here (multiply by 0.0 to 1.0)
        (Sample::SCALE as f32 * signal) as Sample
    }
}

fn parabolic_sin(modulo: f32) -> f32 {
    const B: f32 = 4.0 / PI;
    const C: f32 = -4.0 / (PI * PI);
    const P: f32 = 0.225;

    let angle = PI - modulo * 2.0 * PI;
    let y = B * angle + C * angle * abs(angle);
    P * (y * abs(y) - y) + y
}

#[inline]
fn abs(value: f32) -> f32 {
    if value < 0.0 {
        -value
    } else {
        value
    }
}
