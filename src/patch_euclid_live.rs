use fundsp::prelude32::*;
use crate::nodes::{clock, euclid_live, note_to_frequency, seq};

pub fn build() -> impl AudioUnit {
    // hits modulated by a slow seq; steps fixed per voice
    let gate_a = (clock(100., 16.) | clock(100., 1.) >> seq([3., 5., 3., 7.]) | dc(8.0_f32))  >> euclid_live();
    let gate_b = (clock(100., 16.) | clock(100., 2.) >> seq([5., 3., 7., 2.]) | dc(13.0_f32)) >> euclid_live();
    let gate_c = (clock(100.,  8.) | clock(100., 1.) >> seq([2., 7., 3., 5.]) | dc(11.0_f32)) >> euclid_live();

    (sine_hz(note_to_frequency(60.)) * gate_a
        + sine_hz(note_to_frequency(64.)) * gate_b
        + sine_hz(note_to_frequency(67.)) * gate_c) * 0.2_f32
}
