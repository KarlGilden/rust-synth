//use wasm_bindgen::prelude::*;
use std::f32::consts::{TAU};

// #[wasm_bindgen]
// pub struct Oscillator {
//     freq: f32,
//     sample_rate: f32,
//     phase: f32
// }


// #[wasm_bindgen]
// impl Oscillator {
//     #[wasm_bindgen(constructor)]
//     pub fn new(freq: f32, sample_rate: f32) -> Oscillator {
//         Oscillator { freq, sample_rate, phase: 0.0 }
//     }

//     pub fn next_sample(&mut self) -> f32 {
//         let sample = 2.0 * PI * self.phase.sin();
//         self.phase += self.freq / self.sample_rate;
//         if self.phase >= 1.0 {
//             self.phase -= 1.0;
//         }
//         sample
//     }
// }

// #[wasm_bindgen]
// extern {
//     fn alert(s: &str);
// }

// #[wasm_bindgen]
// pub fn greet(s: &str){
//     alert(s)
// }

#[repr(C)]
pub struct Param {
    current: f32,
    target: f32
}

impl Param {
    fn new(value: f32) -> Self {
        Self {
            current: value,
            target: value,
        }
    }

    fn smooth (&mut self, coeff: f32) {
        self.current += (self.target - self.current) * coeff;
    }
}

#[repr(C)]
pub struct SynthParams {
    pub gain: Param,
    pub frequency: Param
}

#[repr(C)]
#[derive(Copy, Clone)]
enum WaveForm {
    Sine,
    Square,
    Triangle,
    Saw,
}

#[derive(Copy, Clone)]
enum EnvStage {
    Idle,
    Attack,
    Decay,
    Sustain,
    Release
}

pub struct Envelope {
    stage: EnvStage,
    value: f32,

    attack: f32,
    decay: f32,
    sustain: f32,
    release: f32
}

impl Envelope { 
    fn new() -> Self{
        Self {
            stage: EnvStage::Idle, 

            value: 0.0, 
            attack: 0.01, 
            decay: 0.2, 
            sustain: 0.7, 
            release: 0.3 
        }
    }

    fn next(&mut self, sample_rate: f32) -> f32 {
        match self.stage {
            EnvStage::Idle => {
                self.value = 0.0;
            }

            EnvStage::Attack => {
                self.value += 1.0 / (self.attack * sample_rate);
                if self.value >= 1.0 {
                    self.value = 1.0;
                    self.stage = EnvStage::Decay;
                }
            }
            
            EnvStage::Decay => {
                self.value -= (1.0 - self.sustain) / (self.decay * sample_rate);
                if self.value <= self.sustain {
                    self.value = self.sustain;
                    self.stage = EnvStage::Sustain;
                }
            }
            
            EnvStage::Sustain => {

            }

            EnvStage::Release => {
                self.value -= self.sustain / (self.release * sample_rate);
                if self.value <= 0.0 {
                    self.value = 0.0;
                    self.stage = EnvStage::Idle
                }
            }

        }

        self.value
    }

    fn note_on(&mut self){
        self.stage = EnvStage::Attack;
    }

    fn note_off(&mut self){
        self.stage = EnvStage::Release;
    }
}

pub struct Oscillator {
    phase: f32,
    waveform: WaveForm
}

impl Oscillator {
    fn new() -> Self {
        Self {phase: 0.0, waveform: WaveForm::Sine}
    }

    fn sample(&self) -> f32{
        match self.waveform {
            WaveForm::Sine => 
                (self.phase * TAU).sin(),
            
            WaveForm::Square => 
                if self.phase < 0.5 { 1.0 } else { -0.1 },
            
            WaveForm::Triangle => 
                1.0 - 4.0 * (self.phase -0.5).abs(),
            
            WaveForm::Saw => 
                2.0 * self.phase - 1.0
            
        }
    }

    fn next(&mut self, freq: f32, sample_rate: f32) -> f32 {
        let output = self.sample();
        self.phase += freq / sample_rate;

        if self.phase >= 1.0 {
            self.phase -= 1.0;
        }
        output
    }
}

#[repr(C)]
pub struct Synth {
    osc: Oscillator,
    env: Envelope,
    params: SynthParams,
    sample_rate: f32
}

#[no_mangle]
pub extern "C" fn synth_new(sample_rate: f32) -> *mut Synth {
    let synth = Synth{
        osc: Oscillator::new(),
        env: Envelope::new(),
        params: SynthParams { gain: Param::new(0.0), frequency: Param::new(440.0) },
        sample_rate,
    };

    Box::into_raw(Box::new(synth))
}

#[no_mangle]
pub extern "C" fn synth_render(synth: *mut Synth, buffer: *mut f32, frames: usize) {
    let synth = unsafe { &mut *synth };
    let output = unsafe { std::slice::from_raw_parts_mut(buffer, frames)};

    for sample in output.iter_mut() {
        synth.params.gain.smooth(0.001);
        synth.params.frequency.smooth(0.001);

        let env = synth.env.next(synth.sample_rate);
        let osc = synth.osc.next(synth.params.frequency.current, synth.sample_rate);

        let freq = synth.params.frequency.current;
        let loudness_comp = (freq / 440.0).sqrt().clamp(0.5, 1.2);

        *sample = osc * synth.params.gain.current * loudness_comp * env;
    }
}

// Web API

#[no_mangle]
pub extern "C" fn synth_set_gain(synth: *mut Synth, value: f32) {
    unsafe { (*synth).params.gain.target = value}
}

#[no_mangle]
pub extern "C" fn synth_set_frequency(synth: *mut Synth, value: f32) {
    unsafe { (*synth).params.frequency.target = value}
}

#[no_mangle]
pub extern "C" fn synth_note_on(synth: *mut Synth) {
    unsafe { (*synth).env.note_on(); }
}

#[no_mangle]
pub extern "C" fn synth_note_off(synth: *mut Synth) {
    unsafe { (*synth).env.note_off(); }
}

#[no_mangle]
pub extern "C" fn synth_set_adsr(
    synth: *mut Synth,
    attack: f32,
    decay: f32,
    sustain: f32,
    release: f32,
) {
    let env = unsafe { &mut (*synth).env };
    env.attack = attack.max(0.001);
    env.decay = decay.max(0.001);
    env.sustain = sustain.clamp(0.0, 1.0);
    env.release = release.max(0.001);
}

#[no_mangle]
pub extern "C" fn synth_set_waveform(
    synth: *mut Synth,
    waveform: i32,
){
    let osc = unsafe { &mut (*synth).osc };
    osc.waveform = match waveform {
        1 => WaveForm::Square,
        2 => WaveForm::Triangle,
        3 => WaveForm::Saw,
        _ => WaveForm::Sine
    }
}

#[no_mangle]
pub extern "C" fn synth_free(ptr: *mut Synth) {
    unsafe {
        drop(Box::from_raw(ptr));
    }
}

#[no_mangle]
pub extern "C" fn alloc_buffer(frames: usize) -> *mut f32 {
    let mut buffer = Vec::<f32>::with_capacity(frames);
    buffer.resize(frames, 0.0);
    let ptr = buffer.as_mut_ptr();
    std::mem::forget(buffer);
    ptr
}

// #[no_mangle]
// pub extern "C" fn generate_sine(sample_rate: u32, seconds: f32) -> *mut f32 {
//     let total_samples = (sample_rate as f32 * seconds) as usize;
//     let mut buffer = Vec::<f32>::with_capacity(total_samples);

//     let freq = 440.0;
//     let mut phase = 0.0;
//     let phase_inc = freq / sample_rate as f32;

//     for _ in 0..total_samples {
//         buffer.push((phase * TAU).sin());
//         phase += phase_inc;
//         if phase >= 1.0 {
//             phase -= 1.0;
//         }
//     }

//     let ptr = buffer.as_mut_ptr();

//     std::mem::forget(buffer);
//     ptr
// }

// #[no_mangle]
// pub extern "C" fn free_buffer(ptr: *mut f32, len: usize){
//     unsafe{
//         drop(Vec::from_raw_parts(ptr, len, len))
//     }
// }