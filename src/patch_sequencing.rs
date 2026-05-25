use fundsp::prelude32::*;
use crate::nodes::{clock, seq, note_to_hz, pulse, asr_env, pow_n, lin_exp};

fn voice(degrees: &[f64], root: f64) -> An<Unit<U0, U1>> {
    let chord = clock(110., 1.) >> seq([0_f64, -4., -5.]);
    let note  = clock(110., 16.) >> seq(degrees.iter().copied()) + chord + dc(root as f32);
    let freq  = note >> note_to_hz();
    let width = sine_hz(0.1_f32) * 0.225_f32 + 0.275_f32;
    let sq    = (freq | width) >> pulse();
    let fenv  = clock(110., 16.) >> asr_env(0.001, 0.01, 0.1) >> pow_n(3.0) >> lin_exp(0., 1., 100., 5000.);
    unit::<U0, U1>(Box::new(((sq | fenv | dc(0.2_f32)) >> lowpass()) * 0.25_f32))
}

pub fn build() -> impl AudioUnit {
    let bass = voice(&[0., 0., 0., 0., 7., 12., 0., 7.], 36.);
    let mid  = voice(&[3., 2., 0., -2., 0.], 60.);
    let high = voice(&[7., 3., 5., 2., 3., 0.], 72.);

    let henv  = clock(110., 16.) >> asr_env(0.001, 0.01, 0.05);
    let hamp  = clock(110., 16.) >> seq([0.1_f64, 0.25, 0.5, 0.1]);
    let hihat = ((noise() | dc(8000.0_f32) | dc(0.1_f32)) >> highpass()) * henv * hamp;

    (bass + mid + high + hihat) * 0.5_f32
}
