let node;
let ctx;
let notes = {};
let volume = document.getElementById("volume").value;

function pressPower() {
	if (!ctx) {
		start();
	} else {
		stop();
	}

	checkContext();
}

async function start() {
	if (!ctx) {
		ctx = new AudioContext();

		const wasmBytes = await init();

		await ctx.audioWorklet.addModule("synth-processor.js");

		node = new AudioWorkletNode(ctx, "synth-processor", {
			processorOptions: {
				wasmBytes,
			},
		});
		node.connect(ctx.destination);

		if (ctx.state === "suspended") {
			await ctx.resume();
		}
	}
}

function stop() {
	if (!!ctx) {
		ctx = undefined;
		checkContext();
	}
}

function playNote(value) {
	const gain = volume;

	node.port.postMessage({
		type: "gain",
		value: gain / 100,
	});
	node.port.postMessage({
		type: "frequency",
		value: value,
	});
	node.port.postMessage({
		type: "noteOn",
	});
}

function pauseNote() {
	node.port.postMessage({
		type: "noteOff",
	});
}

async function init() {
	const res = await fetch("hello_wasm.wasm");
	return await res.arrayBuffer();
}

function checkContext() {
	if (!ctx) {
		toggleSynthOn(false);
		return;
	}

	toggleSynthOn(true);
}

function toggleSynthOn(isOn) {
	const cover = document.getElementById("synth-blocker");
	const startBtn = document.getElementById("start-btn");
	const powerIcon = document.getElementById("power-icon");
	if (isOn) {
		startBtn.classList.add("highlight-border", "highlight-text");
		powerIcon.classList.add("power-icon-on");
		cover.classList.remove("show");
	} else {
		startBtn.classList.remove("highlight-border", "highlight-text");
		powerIcon.classList.remove("power-icon-on");
		cover.classList.add("show");
	}
}

function onADSRChange() {
	const a = document.getElementById("attack");
	const d = document.getElementById("decay");
	const s = document.getElementById("sustain");
	const r = document.getElementById("release");
	console.log(a.value, d.value, s.value, r.value);
	node.port.postMessage({
		type: "setADSR",
		value: {
			attack: a.value / 100,
			decay: d.value / 100,
			sustain: s.value / 100,
			release: r.value / 100,
		},
	});
}

checkContext();

/**
 * Generate a dictionary of musical note frequencies from startNote to endNote.
 * Uses 12-tone equal temperament tuning with A4 = 440 Hz.
 * @param {string} startNote - e.g., "C0"
 * @param {string} endNote - e.g., "B8"
 * @returns {Object} - { noteName: frequencyHz }
 */
function generateNoteFrequencies(startNote = "C0", endNote = "B8") {
	const noteNames = [
		"C",
		"C#",
		"D",
		"D#",
		"E",
		"F",
		"F#",
		"G",
		"G#",
		"A",
		"A#",
		"B",
	];

	// Convert note name to semitone distance from A4
	function noteToSemitones(note) {
		const name = note.slice(0, -1); // e.g., "C#"
		const octave = parseInt(note.slice(-1), 10);
		const semitoneIndex = noteNames.indexOf(name);
		if (semitoneIndex === -1 || isNaN(octave)) {
			throw new Error(`Invalid note format: ${note}`);
		}
		return (octave - 4) * 12 + (semitoneIndex - noteNames.indexOf("A"));
	}

	const frequencies = {};
	const startIndex = noteToSemitones(startNote);
	const endIndex = noteToSemitones(endNote);

	for (let n = startIndex; n <= endIndex; n++) {
		const freq = 440 * Math.pow(2, n / 12);
		const octave = 4 + Math.floor((n + noteNames.indexOf("A")) / 12);
		const noteName = noteNames[(n + noteNames.indexOf("A")) % 12];
		frequencies[`${noteName}${octave}`] = Math.round(freq * 100) / 100; // round to 2 decimals
	}

	return frequencies;
}

notes = generateNoteFrequencies("C0", "B8");
console.log(
	document.getElementById("attack").value,
	document.getElementById("decay").value,
	document.getElementById("sustain").value,
	document.getElementById("release").value
);
