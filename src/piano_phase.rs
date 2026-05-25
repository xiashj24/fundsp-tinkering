use fundsp::prelude32::*;
use crate::nodes::{clock, fm_voice, note_to_hz, seq};

const PATTERN: [f64; 12] = [-7., -5., 0., 2., 3., -5., -7., 2., 0., -5., 3., 2.];

pub fn build() -> impl AudioUnit {
    // Two voices: same 12-note pattern, voice 2 runs 1% slower → phasing effect
    let bpm = 160.0;
    let voice1 = clock(bpm, 8.0) >> seq(PATTERN) + 60.0 >> note_to_hz() >> fm_voice();
    let voice2 = clock(bpm / 1.01, 8.0) >> seq(PATTERN) + 72.0 >> note_to_hz() >> fm_voice();

    (voice1 | voice2) * 0.25_f32
}
