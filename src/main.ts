import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

let greetInputEl: HTMLInputElement | null;
let greetMsgEl: HTMLElement | null;
let recordBtn: HTMLButtonElement | null;
let stopBtn: HTMLButtonElement | null;
let recordingStatus: HTMLElement | null;

async function greet() {
  if (greetMsgEl && greetInputEl) {
    // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
    greetMsgEl.textContent = await invoke("greet", {
      name: greetInputEl.value,
    });
  }
}

async function startRecording() {
  if (recordBtn && stopBtn && recordingStatus) {
    try {
      recordBtn.disabled = true;
      recordingStatus.textContent = "Recording...";

      // Call the Rust function to start screen recording
      const result = await invoke("start_recording");
      recordingStatus.textContent = result as string;

      // Show the stop button and hide the start button
      recordBtn.style.display = "none";
      stopBtn.style.display = "block";
    } catch (error) {
      recordingStatus.textContent = `Error: ${error}`;
      recordBtn.disabled = false;
    }
  }
}

async function stopRecording() {
  if (recordBtn && stopBtn && recordingStatus) {
    try {
      stopBtn.disabled = true;
      recordingStatus.textContent = "Stopping recording...";

      // Call the Rust function to stop screen recording
      const result = await invoke("stop_recording");
      recordingStatus.textContent = result as string;

      // Show the start button and hide the stop button
      recordBtn.style.display = "block";
      stopBtn.style.display = "none";
    } catch (error) {
      recordingStatus.textContent = `Error: ${error}`;
    } finally {
      stopBtn.disabled = false;
    }
  }
}

// Listen for recording finished event from Rust
listen("recording-finished", (event) => {
  if (recordingStatus) {
    recordingStatus.textContent += ` | ${event.payload}`;
    // Reset buttons after recording is complete
    if (recordBtn && stopBtn) {
      recordBtn.style.display = "block";
      stopBtn.style.display = "none";
      stopBtn.disabled = false;
    }
  }
});

window.addEventListener("DOMContentLoaded", () => {
  greetInputEl = document.querySelector("#greet-input");
  greetMsgEl = document.querySelector("#greet-msg");
  recordBtn = document.querySelector("#record-btn");
  stopBtn = document.querySelector("#stop-btn");
  recordingStatus = document.querySelector("#recording-status");

  document.querySelector("#greet-form")?.addEventListener("submit", (e) => {
    e.preventDefault();
    greet();
  });

  recordBtn?.addEventListener("click", startRecording);
  stopBtn?.addEventListener("click", stopRecording);
});
