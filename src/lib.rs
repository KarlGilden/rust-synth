use std::f32::consts::{TAU};

#[repr(C)]
pub struct Param {
    current: f32,
    target: f32,
    modulation: f32
}

impl Param {
    fn new(value: f32) -> Self {
        Self {
            current: value,
            target: value,
            modulation: 0.0
        }
    }

    fn smooth (&mut self, coeff: f32) {
        self.current += (self.target - self.current) * coeff;
        self.modulation = 0.0;
    }
}

#[repr(C)]
pub enum ParamId {
    Frequency,
    Gain
}

#[repr(C)]
pub enum ModSource {
    Lfo1,
    Env1
}

const SINE_GAIN: f32 = 1.0;
const TRIANGLE_GAIN: f32 = 1.2247;
const SQUARE_GAIN: f32 = 0.7071;
const SAW_GAIN: f32 = 1.2247;

#[repr(C)]
pub struct ModRoute {
    source: ModSource,
    destination: ParamId,
    depth: f32
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

pub struct Lfo {
    phase: f32,
    freq: f32
}

impl Lfo {
    pub fn new() -> Self {
        Self{
            phase: 0.0, 
            freq: 440.0
        }
    }

    pub fn next(&mut self, sample_rate: f32) -> f32 {
        let value = (self.phase * TAU).sin();

        self.phase += self.freq / sample_rate;

        if self.phase >= 1.0 {
            self.phase -= 1.0;
        }

        value
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
                (self.phase * TAU).sin() * SINE_GAIN,
            
            WaveForm::Square => 
                if self.phase < 0.5 { 1.0 * SQUARE_GAIN} else { -0.1 * SQUARE_GAIN },
            
            WaveForm::Triangle => 
                (1.0 - 4.0 * (self.phase -0.5).abs()) * TRIANGLE_GAIN,
            
            WaveForm::Saw => 
                (2.0 * self.phase - 1.0) * SAW_GAIN
            
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
    lfo: Lfo,
    params: [Param; 2],
    routes: Vec<ModRoute>,
    sample_rate: f32
}

#[no_mangle]
pub extern "C" fn synth_new(sample_rate: f32) -> *mut Synth {
    let synth = Synth{
        osc: Oscillator::new(),
        env: Envelope::new(),
        lfo: Lfo::new(),
        params: [
            Param::new(440.0),
            Param::new(0.0),
        ],
        routes: vec![
            ModRoute {
                source: ModSource::Lfo1,
                destination: ParamId::Frequency,
                depth: 1.0
            },
            ModRoute{
                source: ModSource::Env1,
                destination: ParamId::Gain,
                depth: 1.0
            }
        ],
        sample_rate,
    };

    Box::into_raw(Box::new(synth))
}

#[no_mangle]
pub extern "C" fn synth_render(synth: *mut Synth, buffer: *mut f32, frames: usize) {
    let synth = unsafe { &mut *synth };
    let output = unsafe { std::slice::from_raw_parts_mut(buffer, frames)};

    for sample in output.iter_mut() {
        for param in &mut synth.params {
            param.smooth(0.001);
        }

        let lfo_value = synth.lfo.next(synth.sample_rate);
        let env_value = synth.env.next(synth.sample_rate);

        for route in &synth.routes {
            let value = match route.source { 
                ModSource::Lfo1 => lfo_value,
                ModSource::Env1 => env_value
            };

            let destination = match route.destination {
                ParamId::Frequency => ParamId::Frequency as usize,
                ParamId::Gain => ParamId::Gain as usize
            };

            synth.params[destination].modulation += value * route.depth;
        }

        let freq = synth.params[ParamId::Frequency as usize].current + synth.params[ParamId::Frequency as usize].modulation;
        let gain = synth.params[ParamId::Gain as usize].current * synth.params[ParamId::Gain as usize].modulation;

        let osc_value = synth.osc.next(freq.clamp(20.0, 20_000.0), synth.sample_rate);

        *sample = osc_value * gain.clamp(0.0, 1.0) ;
    }
}

// Web API

#[no_mangle]
pub extern "C" fn synth_set_gain(synth: *mut Synth, value: f32) {
    unsafe { (*synth).params[ParamId::Gain as usize] = Param::new(value)}
}

#[no_mangle]
pub extern "C" fn synth_set_frequency(synth: *mut Synth, value: f32) {
    unsafe { (*synth).params[ParamId::Frequency as usize] = Param::new(value)}
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
pub extern "C" fn synth_set_lfo_freq(synth: *mut Synth, freq: f32) {
    unsafe { (*synth).lfo.freq = freq}
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