import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

let greetInputEl: HTMLInputElement | null;
let greetMsgEl: HTMLElement | null;
let recordBtn: HTMLButtonElement | null;
let stopBtn: HTMLButtonElement | null;
let screenshotStatus: HTMLElement | null;

async function greet() {
  if (greetMsgEl && greetInputEl) {
    // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
    greetMsgEl.textContent = await invoke("greet", {
      name: greetInputEl.value,
    });
  }
}

async function startCombinedRecording() {
  if (recordBtn && stopBtn && screenshotStatus) {
    try {
      recordBtn.disabled = true;
      screenshotStatus.textContent = "Remote Worker: Starting...";

      // Call the Rust function to start combined recording
      const result = await invoke("start_combined_recording");
      screenshotStatus.textContent = result as string;

      // Show the stop button and hide the record button
      recordBtn.style.display = "none";
      stopBtn.style.display = "block";
      stopBtn.disabled = false; // Ensure stop button is enabled
    } catch (error) {
      screenshotStatus.textContent = `Error: ${error}`;
      recordBtn.disabled = false;
      // Show the record button again if there's an error
      recordBtn.style.display = "block";
      stopBtn.style.display = "none";
    }
  }
}

async function stopCombinedRecording() {
  if (stopBtn && screenshotStatus) {
    try {
      stopBtn.disabled = true;
      screenshotStatus.textContent = "Stopping recording...";

      // Call the Rust function to stop combined recording
      const result = await invoke("stop_combined_recording");
      screenshotStatus.textContent = result as string;

    } catch (error) {
      screenshotStatus.textContent = `Error: ${error}`;
    } finally {
      // Always reset the UI regardless of whether the Rust call succeeded
      // Show the record button and hide the stop button
      recordBtn.style.display = "block";
      stopBtn.style.display = "none";
      stopBtn.disabled = false;
      recordBtn.disabled = false;  // Ensure button is enabled
    }
  }
}

// Listen for screenshot taken event from Rust
listen("screenshot-taken", (event) => {
  if (screenshotStatus) {
    screenshotStatus.textContent = `Screenshot taken: ${event.payload}`;
  }
});

// Listen for screenshotting finished event from Rust
listen("screenshotting-finished", (event) => {
  if (screenshotStatus) {
    screenshotStatus.textContent += ` | ${event.payload}`;
    // Reset buttons after screenshotting is stopped
    if (screenshotBtn && stopBtn) {
      screenshotBtn.style.display = "block";
      stopBtn.style.display = "none";
      stopBtn.disabled = false;
      screenshotBtn.disabled = false; // Ensure start button is not disabled
    }
  }
});

// Listen for recording events from Rust
listen("recording-started", (event) => {
  if (screenshotStatus) {
    screenshotStatus.textContent = `Recording started: ${event.payload}`;
  }
});

listen("recording-progress", (event) => {
  if (screenshotStatus) {
    screenshotStatus.textContent = `Recording progress: ${event.payload}`;
  }
});

listen("recording-finished", (event) => {
  if (screenshotStatus) {
    screenshotStatus.textContent = `Recording finished: ${event.payload}`;
    // Reset buttons after recording is stopped
    if (recordBtn && stopBtn) {
      recordBtn.style.display = "block";
      stopBtn.style.display = "none";
      stopBtn.disabled = false;
      recordBtn.disabled = false; // Ensure start button is not disabled
    }
  }
});

listen("recording-converted", (event) => {
  if (screenshotStatus) {
    screenshotStatus.textContent = `Video created: ${event.payload}`;
  }
});

listen("recording-error", (event) => {
  if (screenshotStatus) {
    screenshotStatus.textContent = `Recording error: ${event.payload}`;
  }
});

window.addEventListener("DOMContentLoaded", () => {
  greetInputEl = document.querySelector("#greet-input");
  greetMsgEl = document.querySelector("#greet-msg");
  recordBtn = document.querySelector("#record-btn");
  stopBtn = document.querySelector("#stop-btn");
  screenshotStatus = document.querySelector("#screenshot-status");

  document.querySelector("#greet-form")?.addEventListener("submit", (e) => {
    e.preventDefault();
    greet();
  });

  recordBtn?.addEventListener("click", startCombinedRecording);
  stopBtn?.addEventListener("click", stopCombinedRecording);

  // Listen for progress updates from Rust (just for time display)
  listen("recording-progress", (event) => {
    if (typeof event.payload === 'string' && event.payload.includes("Next snapshot in:")) {
      // Just update the status text with the remaining time
      const payloadParts = event.payload.split('|');
      const timePart = payloadParts[0];
      if (screenshotStatus) {
        screenshotStatus.textContent = timePart;
      }
    }
  });
});
