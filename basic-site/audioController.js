function setGain(value) {
	node.port.postMessage({
		type: "gain",
		value: value,
	});
}

function setFrequency(value) {
	node.port.postMessage({
		type: "frequency",
		value: value,
	});
}

function noteOn(value) {
	node.port.postMessage({
		type: "noteOn",
	});
}

function noteOff() {
	node.port.postMessage({
		type: "noteOff",
	});
}

function setEnvelope(a, d, s, r) {
	node.port.postMessage({
		type: "envelope",
		value: {
			attack: a,
			decay: d,
			sustain: s,
			release: r,
		},
	});
}

function setLFOFrequency(value) {
	node.port.postMessage({
		type: "lfoFreq",
		value: value,
	});
}

function setWaveform(value) {
	node.port.postMessage({
		type: "waveform",
		value: value,
	});
}
