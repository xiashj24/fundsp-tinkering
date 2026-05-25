use fundsp::prelude32::*;
use crate::nodes::{clock, euclid, note_to_frequency};

pub fn build() -> impl AudioUnit {
    (sine_hz(note_to_frequency(60.)) * (clock(100., 16.) >> euclid(5,  8))
        + sine_hz(note_to_frequency(62.)) * (clock(100.,  8.) >> euclid(5, 13))
        + sine_hz(note_to_frequency(64.)) * (clock(100.,  8.) >> euclid(7, 15))
        + sine_hz(note_to_frequency(67.)) * (clock(100., 16.) >> euclid(6, 19))
        + sine_hz(note_to_frequency(71.)) * (clock(100.,  8.) >> euclid(7, 23)))
        * 0.15_f32
}
