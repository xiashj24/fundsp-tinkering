use fundsp::prelude32::*;

// SC: 2**LFNoise0.kr(4/3, 4)*300
// Stepped random at 4/3 Hz with range [-4, 4] mapped through 2^x * 300
// → frequency sweeps between 18.75 Hz and 4800 Hz in discrete steps
fn bpf_freq(t: f32) -> f32 {
    let step = (t * 4.0 / 3.0) as u64;
    (2.0_f64.powf((rnd1(step) * 2.0 - 1.0) * 4.0) * 300.0) as f32
}

pub fn build() -> impl AudioUnit {
    // SC: Saw.ar([32, 33], 0.2)
    let saws = saw_hz(32.0) * 0.2 | saw_hz(33.0) * 0.2;

    // SC: BPF.ar(..., freq, rq=0.1) — rq is reciprocal Q, so Q = 10
    // Two identical BPF instances share the same deterministic LFO function
    let bpf_stereo =
        ((pass() | lfo(bpf_freq) | dc(10.0)) >> bandpass()) |
        ((pass() | lfo(bpf_freq) | dc(10.0)) >> bandpass());

    // SC: .distort = x / (1 + |x|) — Softsign(1.0) is exactly this
    let distort_stereo = shape(Softsign(1.0_f32)) | shape(Softsign(1.0_f32));

    // SC: CombN.ar(in, maxDelay=2, delay=2, decayTime=40)
    // IIR comb: y[n] ≈ x[n] + coeff * y[n-D], coeff = 0.001^(delay/decay)
    let comb_coeff = 0.001_f64.powf(2.0 / 40.0) as f32; // ≈ 0.708
    let comb_stereo =
        multipass::<U2>() & feedback((delay(2.0) | delay(2.0)) * comb_coeff);

    // BPF → distort → comb (stereo 2→2)
    let process = bpf_stereo >> distort_stereo >> comb_stereo;

    // SC: LocalIn.ar(2)*7.5 + Saw — outer feedback loop with 7.5x gain
    // feedback2(x, y): output[n] = x(input[n] + y(output[n-1]))
    let drone = saws >> feedback2(process, pass() * 7.5 | pass() * 7.5);

    drone >> (dcblock() | dcblock()) >> limiter_stereo(1.0, 5.0)
}
