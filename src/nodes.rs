use fundsp::prelude32::*;
use std::f32::consts::TAU;

pub fn note_to_frequency(note: f64) -> f32 {
    (440.0 * 2.0_f64.powf((note - 69.0) / 12.0)) as f32
}

// ── ClockTrigger ─────────────────────────────────────────────────────────────
// Fires a 1-sample trigger impulse at each step boundary, 0.0 otherwise.

#[derive(Clone)]
pub struct ClockTrigger {
    bpm: f64,
    subdivisions: f64,
    elapsed: f64,
    samples_per_step: f64,
}

impl AudioNode for ClockTrigger {
    const ID: u64 = 1001;
    type Inputs = U0;
    type Outputs = U1;

    fn set_sample_rate(&mut self, sr: f64) {
        // subdivisions = steps per whole note (automata convention: 4=quarter, 8=eighth, 16=sixteenth)
        // whole notes per second = (bpm / 60) / 4
        self.samples_per_step = sr / (self.bpm / 60.0 / 4.0 * self.subdivisions);
    }

    fn tick(&mut self, _: &Frame<f32, Self::Inputs>) -> Frame<f32, Self::Outputs> {
        self.elapsed += 1.0;
        if self.elapsed >= self.samples_per_step {
            self.elapsed -= self.samples_per_step;
            [1.0_f32].into()
        } else {
            [0.0_f32].into()
        }
    }

    fn reset(&mut self) {
        self.elapsed = 0.0;
    }
}

pub fn clock(bpm: f64, subdivisions: f64) -> An<ClockTrigger> {
    An(ClockTrigger {
        bpm,
        subdivisions,
        elapsed: 0.0,
        samples_per_step: DEFAULT_SR / (bpm / 60.0 / 4.0 * subdivisions),
    })
}

// ── Euclid ───────────────────────────────────────────────────────────────────
// Euclidean rhythm gate: outputs 1.0 on hit steps, 0.0 otherwise.

fn euclidean_pattern(hits: usize, steps: usize) -> Vec<bool> {
    let mut pattern = vec![false; steps];
    let mut bucket = 0;
    for i in 0..steps {
        bucket += hits;
        if bucket >= steps {
            bucket -= steps;
            pattern[i] = true;
        }
    }
    pattern
}

#[derive(Clone)]
pub struct Euclid {
    pattern: Vec<bool>,
    index: usize,
    gate: f32,
}

impl AudioNode for Euclid {
    const ID: u64 = 1005;
    type Inputs = U1;
    type Outputs = U1;

    fn tick(&mut self, input: &Frame<f32, Self::Inputs>) -> Frame<f32, Self::Outputs> {
        if input[0] > 0.5 {
            self.index = (self.index + 1) % self.pattern.len();
            self.gate = if self.pattern[self.index] { 1.0 } else { 0.0 };
        }
        [self.gate].into()
    }

    fn reset(&mut self) {
        self.index = 0;
        self.gate = if self.pattern[0] { 1.0 } else { 0.0 };
    }
}

pub fn euclid(hits: usize, steps: usize) -> An<Euclid> {
    let pattern = euclidean_pattern(hits, steps);
    let gate = if pattern[0] { 1.0 } else { 0.0 };
    An(Euclid { pattern, index: 0, gate })
}

// ── EuclidLive ───────────────────────────────────────────────────────────────
// Euclidean gate with modulatable hits and steps (3 inputs: trigger | hits | steps).
// Runs Bresenham live so hits/steps can be signals.

#[derive(Clone, Default)]
pub struct EuclidLive {
    bucket: f32,
    gate: f32,
}

impl AudioNode for EuclidLive {
    const ID: u64 = 1006;
    type Inputs = U3;
    type Outputs = U1;

    fn tick(&mut self, input: &Frame<f32, Self::Inputs>) -> Frame<f32, Self::Outputs> {
        if input[0] > 0.5 {
            self.bucket += input[1];
            if self.bucket >= input[2] {
                self.bucket -= input[2];
                self.gate = 1.0;
            } else {
                self.gate = 0.0;
            }
        }
        [self.gate].into()
    }

    fn reset(&mut self) {
        self.bucket = 0.0;
        self.gate = 0.0;
    }
}

pub fn euclid_live() -> An<EuclidLive> {
    An(EuclidLive::default())
}

// ── StepSeq ──────────────────────────────────────────────────────────────────
// Cycles through a note table (Hz), advancing one step per trigger impulse.

#[derive(Clone)]
pub struct StepSeq {
    notes: Vec<f32>,
    index: usize,
}

impl AudioNode for StepSeq {
    const ID: u64 = 1002;
    type Inputs = U1;
    type Outputs = U1;

    fn tick(&mut self, input: &Frame<f32, Self::Inputs>) -> Frame<f32, Self::Outputs> {
        if input[0] > 0.5 {
            self.index = (self.index + 1) % self.notes.len();
        }
        [self.notes[self.index]].into()
    }

    fn reset(&mut self) {
        self.index = 0;
    }
}

pub fn seq(notes: impl IntoIterator<Item = f64>) -> An<StepSeq> {
    let notes: Vec<f32> = notes.into_iter().map(|n| n as f32).collect();
    An(StepSeq { notes, index: 0 })
}

// ── NoteToHz ─────────────────────────────────────────────────────────────────
// Converts a MIDI note number signal to Hz.

#[derive(Clone, Default)]
pub struct NoteToHz;

impl AudioNode for NoteToHz {
    const ID: u64 = 1004;
    type Inputs = U1;
    type Outputs = U1;

    fn tick(&mut self, input: &Frame<f32, Self::Inputs>) -> Frame<f32, Self::Outputs> {
        [note_to_frequency(input[0] as f64)].into()
    }
}

pub fn note_to_hz() -> An<NoteToHz> {
    An(NoteToHz)
}

// ── FmVoice ──────────────────────────────────────────────────────────────────
// 1:1 FM synthesis: output = sin(2π·phase + sin(2π·phase))
// Matches automata's simple_fm(freq, index=1).

#[derive(Clone)]
pub struct FmVoice {
    phase: f32,
    sample_rate: f32,
}

impl AudioNode for FmVoice {
    const ID: u64 = 1003;
    type Inputs = U1;
    type Outputs = U1;

    fn set_sample_rate(&mut self, sr: f64) {
        self.sample_rate = sr as f32;
    }

    fn reset(&mut self) {
        self.phase = 0.0;
    }

    fn tick(&mut self, input: &Frame<f32, Self::Inputs>) -> Frame<f32, Self::Outputs> {
        let freq = input[0];
        let angle = self.phase * TAU;
        let sample = (angle + angle.sin()).sin();
        self.phase = (self.phase + freq / self.sample_rate).fract();
        [sample].into()
    }
}

pub fn fm_voice() -> An<FmVoice> {
    An(FmVoice {
        phase: 0.0,
        sample_rate: DEFAULT_SR as f32,
    })
}

// ── Pulse ─────────────────────────────────────────────────────────────────────
// Variable-width pulse oscillator. Inputs: freq (Hz), width (0..1).

#[derive(Clone)]
pub struct Pulse {
    phase: f32,
    sample_rate: f32,
}

impl AudioNode for Pulse {
    const ID: u64 = 1007;
    type Inputs = U2;
    type Outputs = U1;

    fn set_sample_rate(&mut self, sr: f64) {
        self.sample_rate = sr as f32;
    }

    fn reset(&mut self) {
        self.phase = 0.0;
    }

    fn tick(&mut self, input: &Frame<f32, Self::Inputs>) -> Frame<f32, Self::Outputs> {
        let freq = input[0];
        let width = input[1].clamp(0.0, 1.0);
        let sample = if self.phase < width { 1.0_f32 } else { -1.0_f32 };
        self.phase = (self.phase + freq / self.sample_rate).fract();
        [sample].into()
    }
}

pub fn pulse() -> An<Pulse> {
    An(Pulse { phase: 0.0, sample_rate: DEFAULT_SR as f32 })
}

// ── AsrEnv ────────────────────────────────────────────────────────────────────
// One-shot ASR envelope. Input: trigger impulse. Fires attack→sustain→release.

#[derive(Clone, PartialEq)]
enum AsrPhase { Idle, Attack, Sustain, Release }

#[derive(Clone)]
pub struct AsrEnv {
    attack_s: f32,
    sustain_s: f32,
    release_s: f32,
    phase: AsrPhase,
    elapsed: f32,
    sample_rate: f32,
}

impl AudioNode for AsrEnv {
    const ID: u64 = 1008;
    type Inputs = U1;
    type Outputs = U1;

    fn set_sample_rate(&mut self, sr: f64) {
        self.sample_rate = sr as f32;
    }

    fn reset(&mut self) {
        self.phase = AsrPhase::Idle;
        self.elapsed = 0.0;
    }

    fn tick(&mut self, input: &Frame<f32, Self::Inputs>) -> Frame<f32, Self::Outputs> {
        if input[0] > 0.5 {
            self.phase = AsrPhase::Attack;
            self.elapsed = 0.0;
        }
        let output = match self.phase {
            AsrPhase::Idle => 0.0,
            AsrPhase::Attack => {
                let total = (self.attack_s * self.sample_rate).max(1.0);
                let v = (self.elapsed / total).min(1.0);
                self.elapsed += 1.0;
                if self.elapsed >= total {
                    self.phase = AsrPhase::Sustain;
                    self.elapsed = 0.0;
                }
                v
            }
            AsrPhase::Sustain => {
                self.elapsed += 1.0;
                if self.elapsed >= (self.sustain_s * self.sample_rate).max(1.0) {
                    self.phase = AsrPhase::Release;
                    self.elapsed = 0.0;
                }
                1.0
            }
            AsrPhase::Release => {
                let total = (self.release_s * self.sample_rate).max(1.0);
                let v = (1.0 - self.elapsed / total).max(0.0);
                self.elapsed += 1.0;
                if self.elapsed >= total {
                    self.phase = AsrPhase::Idle;
                    self.elapsed = 0.0;
                }
                v
            }
        };
        [output].into()
    }
}

pub fn asr_env(attack_s: f32, sustain_s: f32, release_s: f32) -> An<AsrEnv> {
    An(AsrEnv {
        attack_s,
        sustain_s,
        release_s,
        phase: AsrPhase::Idle,
        elapsed: 0.0,
        sample_rate: DEFAULT_SR as f32,
    })
}

// ── PowF ──────────────────────────────────────────────────────────────────────
// Raises input to a fixed power. Constructor: pow_n(exponent).

#[derive(Clone)]
pub struct PowF {
    exponent: f32,
}

impl AudioNode for PowF {
    const ID: u64 = 1009;
    type Inputs = U1;
    type Outputs = U1;

    fn tick(&mut self, input: &Frame<f32, Self::Inputs>) -> Frame<f32, Self::Outputs> {
        [input[0].powf(self.exponent)].into()
    }
}

pub fn pow_n(n: f32) -> An<PowF> {
    An(PowF { exponent: n })
}

// ── LinExp ────────────────────────────────────────────────────────────────────
// Maps linear [in0, in1] to exponential [out0, out1].
// Formula: out0 * (out1/out0)^((x - in0) / (in1 - in0))

#[derive(Clone)]
pub struct LinExp {
    in0: f32,
    in1: f32,
    out0: f32,
    out1: f32,
}

impl AudioNode for LinExp {
    const ID: u64 = 1010;
    type Inputs = U1;
    type Outputs = U1;

    fn tick(&mut self, input: &Frame<f32, Self::Inputs>) -> Frame<f32, Self::Outputs> {
        let t = (input[0] - self.in0) / (self.in1 - self.in0);
        [(self.out0 * (self.out1 / self.out0).powf(t))].into()
    }
}

pub fn lin_exp(in0: f32, in1: f32, out0: f32, out1: f32) -> An<LinExp> {
    An(LinExp { in0, in1, out0, out1 })
}
