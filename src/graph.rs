use fundsp::prelude64::*;

pub fn build() -> impl AudioUnit {
    let signal = sine_hz(440.0) * 0.3 >> lowpole_hz(800.0) >> pan(0.0);
    signal >> (declick() | declick()) >> limiter_stereo(1.0, 5.0)
}
