import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { NetworkUsageComponent } from "./network-usage";

let recordBtn: HTMLButtonElement | null;
let stopBtn: HTMLButtonElement | null;
let screenshotStatus: HTMLElement | null;
let activityBadge: HTMLElement | null;

async function startCombinedRecording() {
  if (recordBtn && stopBtn && screenshotStatus) {
    try {
      recordBtn.disabled = true;
      screenshotStatus.textContent = "Remote Worker: Starting...";

      // Call the Rust function to start combined recording
      const result = await invoke("start_combined_recording");
      screenshotStatus.textContent = result as string;

      // Start idle detection
      await invoke("start_idle_detection");

      // Update badge to recording state (blue)
      if (activityBadge) {
        activityBadge.style.backgroundColor = '#2196F3'; // Blue color for recording
        activityBadge.style.boxShadow = '0 0 10px rgba(33, 150, 243, 0.7)'; // Blue glow for recording
      }

      // Show the stop button and hide the record button
      if (recordBtn) recordBtn.style.display = "none";
      if (stopBtn) {
        stopBtn.style.display = "block";
        stopBtn.disabled = false; // Ensure stop button is enabled
      }
    } catch (error) {
      screenshotStatus.textContent = `Error: ${error}`;
      if (recordBtn) recordBtn.disabled = false;
      // Show the record button again if there's an error
      if (recordBtn) recordBtn.style.display = "block";
      if (stopBtn) stopBtn.style.display = "none";
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

      // Stop idle detection
      await invoke("stop_idle_detection");

      // Update badge to stopped state (red)
      if (activityBadge) {
        activityBadge.style.backgroundColor = '#f44336'; // Red color for stopped
        activityBadge.style.boxShadow = '0 0 10px rgba(244, 67, 54, 0.7)'; // Red glow for stopped
      }

    } catch (error) {
      screenshotStatus.textContent = `Error: ${error}`;
    } finally {
      // Always reset the UI regardless of whether the Rust call succeeded
      // Show the record button and hide the stop button
      if (recordBtn) {
        recordBtn.style.display = "block";
        recordBtn.disabled = false;  // Ensure button is enabled
      }
      if (stopBtn) stopBtn.style.display = "none";
      if (stopBtn) stopBtn.disabled = false;
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
    if (recordBtn && stopBtn) {
      if (recordBtn) {
        recordBtn.style.display = "block";
        recordBtn.disabled = false; // Ensure start button is not disabled
      }
      if (stopBtn) {
        stopBtn.style.display = "none";
        stopBtn.disabled = false;
      }
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
      if (recordBtn) {
        recordBtn.style.display = "block";
        recordBtn.disabled = false; // Ensure start button is not disabled
      }
      if (stopBtn) {
        stopBtn.style.display = "none";
        stopBtn.disabled = false;
      }
    }
  }
  // Update activity badge to stopped state
  if (activityBadge) {
    activityBadge.style.backgroundColor = '#f44336'; // Red color for stopped
    activityBadge.style.boxShadow = '0 0 10px rgba(244, 67, 54, 0.7)'; // Red glow for stopped
  }
});

listen("recording-paused", (event) => {
  if (screenshotStatus) {
    screenshotStatus.textContent = `Recording paused: ${event.payload}`;
  }
  // Update UI to reflect paused state (the recording is still "running" in a paused state)
  // The stop button should remain visible but we should indicate it's paused
  if (activityBadge) {
    activityBadge.style.backgroundColor = '#FFC107'; // Yellow color for paused
    activityBadge.style.boxShadow = '0 0 10px rgba(255, 193, 7, 0.7)'; // Yellow glow for paused
  }
});

listen("recording-resumed", (event) => {
  if (screenshotStatus) {
    screenshotStatus.textContent = `Recording resumed: ${event.payload}`;
  }
  // Update UI to reflect resumed state
  if (activityBadge) {
    activityBadge.style.backgroundColor = '#2196F3'; // Blue color for recording
    activityBadge.style.boxShadow = '0 0 10px rgba(33, 150, 243, 0.7)'; // Blue glow for recording
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

// Listen for all processes stopped event from Rust (when stopped from admin panel)
listen("all-processes-stopped", (event) => {
  if (screenshotStatus) {
    screenshotStatus.textContent = `All processes stopped: ${event.payload}`;
    // Reset buttons after all processes are stopped
    if (recordBtn && stopBtn) {
      if (recordBtn) {
        recordBtn.style.display = "block";
        recordBtn.disabled = false; // Ensure start button is not disabled
      }
      if (stopBtn) {
        stopBtn.style.display = "none";
        stopBtn.disabled = false;
      }
    }
  }
  // Update activity badge to stopped state
  if (activityBadge) {
    activityBadge.style.backgroundColor = '#f44336'; // Red color for stopped
    activityBadge.style.boxShadow = '0 0 10px rgba(244, 67, 54, 0.7)'; // Red glow for stopped
  }
});

window.addEventListener("DOMContentLoaded", () => {
  recordBtn = document.querySelector("#record-btn");
  stopBtn = document.querySelector("#stop-btn");
  screenshotStatus = document.querySelector("#screenshot-status");
  activityBadge = document.querySelector("#activity-badge");

  recordBtn?.addEventListener("click", startCombinedRecording);
  stopBtn?.addEventListener("click", stopCombinedRecording);

  // Update user activity on any user interaction
  document.addEventListener('mousemove', () => {
    invoke('update_user_activity');
  });

  document.addEventListener('keydown', () => {
    invoke('update_user_activity');
  });

  document.addEventListener('click', () => {
    invoke('update_user_activity');
  });

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

  // Listen for idle/active status updates from Rust
  listen("user-idle", (event) => {
    if (typeof event.payload === 'string') {
      if (screenshotStatus) {
        screenshotStatus.textContent = `Idle: ${event.payload}`;
      }
      if (activityBadge) {
        activityBadge.style.backgroundColor = '#FFC107'; // Yellow color for idle
        activityBadge.style.boxShadow = '0 0 10px rgba(255, 193, 7, 0.7)'; // Yellow glow for idle
      }
    }
  });

  listen("user-active", (event) => {
    if (typeof event.payload === 'string') {
      if (screenshotStatus) {
        screenshotStatus.textContent = `Active: ${event.payload}`;
      }
      if (activityBadge) {
        activityBadge.style.backgroundColor = '#4CAF50'; // Green color for active
        activityBadge.style.boxShadow = '0 0 10px rgba(76, 175, 80, 0.7)'; // Green glow for active
      }
    }
  });

  // Add keyboard shortcut for admin window (Ctrl+Shift+A)
  document.addEventListener('keydown', (event) => {
    if (event.ctrlKey && event.shiftKey && event.key === 'a') {
      event.preventDefault(); // Prevent default action
      createAdminWindow();
    }
  });

  // Initialize network usage component (with global network stats)
  const networkUsage = new NetworkUsageComponent(true);
  networkUsage.start();

  async function createAdminWindow() {
    try {
      // Call Rust function to create admin window
      await invoke("create_admin_window");
    } catch (error) {
      console.error("Error creating admin window:", error);
    }
  }

  // Removed admin button listener since button no longer exists in DOM
});
