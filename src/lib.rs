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

pub struct Oscillator {
    phase: f32,
}

impl Oscillator {
    fn new() -> Self {
        Self {phase: 0.0}
    }

    fn next(&mut self, freq: f32, sample_rate: f32) -> f32 {
        let output = (self.phase * TAU).sin();
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
    params: SynthParams,
    sample_rate: f32
}

#[no_mangle]
pub extern "C" fn synth_new(sample_rate: f32) -> *mut Synth {
    let synth = Synth{
        osc: Oscillator::new(),
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

        let osc = synth.osc.next(synth.params.frequency.current, synth.sample_rate);

        let freq = synth.params.frequency.current;
        let loudness_comp = (freq / 440.0).sqrt().clamp(0.5, 1.2);

        *sample = osc * synth.params.gain.current * loudness_comp;
    }
}

#[no_mangle]
pub extern "C" fn synth_set_gain(synth: *mut Synth, value: f32) {
    unsafe { (*synth).params.gain.target = value}
}

#[no_mangle]
pub extern "C" fn synth_set_frequency(synth: *mut Synth, value: f32) {
    unsafe { (*synth).params.frequency.target = value}
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