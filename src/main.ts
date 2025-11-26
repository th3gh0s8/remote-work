import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

let greetInputEl: HTMLInputElement | null;
let greetMsgEl: HTMLElement | null;
let screenshotBtn: HTMLButtonElement | null;
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

async function startScreenshotting() {
  if (screenshotBtn && stopBtn && screenshotStatus) {
    try {
      screenshotBtn.disabled = true;
      screenshotStatus.textContent = "Screenshotting started... Screenshots will be taken every 15 minutes";

      // Call the Rust function to start scheduled screenshotting
      const result = await invoke("start_screenshotting");
      screenshotStatus.textContent = result as string;

      // Show the stop button and hide the start button
      screenshotBtn.style.display = "none";
      stopBtn.style.display = "block";
    } catch (error) {
      screenshotStatus.textContent = `Error: ${error}`;
      screenshotBtn.disabled = false;
    }
  }
}

async function stopScreenshotting() {
  if (screenshotBtn && stopBtn && screenshotStatus) {
    try {
      stopBtn.disabled = true;
      screenshotStatus.textContent = "Stopping screenshotting...";

      // Call the Rust function to stop scheduled screenshotting
      const result = await invoke("stop_screenshotting");
      screenshotStatus.textContent = result as string;

      // Show the start button and hide the stop button
      screenshotBtn.style.display = "block";
      stopBtn.style.display = "none";
    } catch (error) {
      screenshotStatus.textContent = `Error: ${error}`;
    } finally {
      stopBtn.disabled = false;
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
    }
  }
});

window.addEventListener("DOMContentLoaded", () => {
  greetInputEl = document.querySelector("#greet-input");
  greetMsgEl = document.querySelector("#greet-msg");
  screenshotBtn = document.querySelector("#screenshot-btn");
  stopBtn = document.querySelector("#stop-btn");
  screenshotStatus = document.querySelector("#screenshot-status");

  document.querySelector("#greet-form")?.addEventListener("submit", (e) => {
    e.preventDefault();
    greet();
  });

  screenshotBtn?.addEventListener("click", startScreenshotting);
  stopBtn?.addEventListener("click", stopScreenshotting);
});
