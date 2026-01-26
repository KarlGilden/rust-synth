class SynthProcessor extends AudioWorkletProcessor {
	constructor(options) {
		super();

		this.playing = false;
		this.ready = false;

		this.initWasm(options);
	}

	async initWasm(options) {
		const wasmBytes = options.processorOptions.wasmBytes;

		WebAssembly.instantiate(wasmBytes, {}).then(({ instance }) => {
			this.wasm = instance.exports;
			this.memory = this.wasm.memory;

			this.synth = this.wasm.synth_new(sampleRate);

			let frames = 128;
			this.bufferPtr = this.wasm.alloc_buffer(frames);

			this.buffer = new Float32Array(
				this.memory.buffer,

				this.bufferPtr,
				frames
			);
			this.ready = true;
		});

		this.port.onmessage = e => {
			const { type, value } = e.data;
			console.log(value);
			if (type === "gain") {
				this.wasm.synth_set_gain(this.synth, value);
			}

			if (type === "frequency") {
				this.wasm.synth_set_frequency(this.synth, value);
			}

			if (type === "noteOn") {
				this.wasm.synth_note_on(this.synth);
			}

			if (type === "noteOff") {
				this.wasm.synth_note_off(this.synth);
			}

			if (type === "setADSR") {
				this.wasm.synth_set_adsr(
					this.synth,
					value.attack,
					value.decay,
					value.sustain,
					value.release
				);
			}
		};
	}

	process(_, outputs) {
		const output = outputs[0][0];

		const frames = output.length;

		if (this.buffer.length !== frames) {
			this.buffer = new Float32Array(frames);
		}

		this.wasm.synth_render(this.synth, this.buffer.byteOffset, frames);

		output.set(this.buffer);

		return true;
	}
}

registerProcessor("synth-processor", SynthProcessor);
