import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { NetworkUsageComponent } from "./network-usage";

// Check if user ID is set, if not show welcome UI
async function checkUserId() {
  try {
    const isSet = await invoke('is_user_id_set');
    if (!isSet) {
      // Show welcome UI instead of redirecting
      showWelcomeScreen();
    } else {
      // User ID is set, show main app UI
      showMainApp();
    }
  } catch (error) {
    console.error('Error checking user ID:', error);
    // Show welcome UI as fallback
    showWelcomeScreen();
  }
}

// Store original HTML content
let originalHTML: string | null = null;

// Function to display welcome UI
function showWelcomeScreen() {
  // Store the original HTML on first visit (if not already stored)
  if (!originalHTML) {
    originalHTML = document.documentElement.innerHTML;
  }

  const welcomeHTML = `
    <div class="welcome-container" style="
      position: fixed;
      top: 0;
      left: 0;
      width: 100%;
      height: 100%;
      background: linear-gradient(135deg, #1a2a6c, #b21f1f, #1a2a6c);
      display: flex;
      justify-content: center;
      align-items: center;
      z-index: 10000;
      color: white;
      font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
    ">
      <div class="welcome-content" style="
        background: rgba(0, 0, 0, 0.7);
        border-radius: 15px;
        padding: 40px;
        width: 400px;
        text-align: center;
        box-shadow: 0 10px 30px rgba(0, 0, 0, 0.5);
        backdrop-filter: blur(10px);
      ">
        <div class="logo" style="
          width: 100px;
          height: 100px;
          margin: 0 auto 20px;
          display: flex;
          align-items: center;
          justify-content: center;
          background: rgba(76, 175, 80, 0.2);
          border-radius: 50%;
          font-size: 2em;
          border: 3px solid #4CAF50;
          margin-bottom: 30px;
        ">RW</div>
        <h1 style="
          margin-top: 0;
          color: #4CAF50;
          font-size: 2.5em;
          text-shadow: 0 0 10px rgba(76, 175, 80, 0.5);
        ">Welcome!</h1>
        <p style="
          font-size: 1.1em;
          margin-bottom: 30px;
          color: #ddd;
        ">Please enter your User ID to continue</p>

        <div class="input-group" style="
          margin-bottom: 20px;
          text-align: left;
        ">
          <label for="userId" style="
            display: block;
            margin-bottom: 8px;
            font-weight: bold;
            color: #4CAF50;
          ">User ID:</label>
          <input type="text" id="userId" placeholder="Enter your User ID" autocomplete="off" style="
            width: 100%;
            padding: 12px;
            border: 2px solid #4CAF50;
            border-radius: 8px;
            background: rgba(255, 255, 255, 0.1);
            color: white;
            font-size: 16px;
            box-sizing: border-box;
          ">
        </div>

        <button id="continueBtn" style="
          background: #4CAF50;
          color: white;
          border: none;
          padding: 15px 30px;
          font-size: 18px;
          border-radius: 8px;
          cursor: pointer;
          width: 100%;
          margin-top: 10px;
          transition: all 0.3s ease;
        ">Continue</button>

        <div id="statusMessage" class="status-message" style="
          margin-top: 15px;
          padding: 10px;
          border-radius: 5px;
          display: none;
        "></div>
      </div>
    </div>
  `;
  
  document.body.innerHTML = welcomeHTML;
  
  // Add event listeners for the welcome screen
  const userIdInput = document.getElementById('userId') as HTMLInputElement;
  const continueBtn = document.getElementById('continueBtn') as HTMLButtonElement;
  const statusMessage = document.getElementById('statusMessage') as HTMLDivElement;

  // Set up continue button event listener
  continueBtn.addEventListener('click', async () => {
    const userId = userIdInput.value.trim();

    if (!userId) {
      showStatusMessage(statusMessage, 'Please enter a User ID', 'error');
      return;
    }

    // Validate user ID format
    if (userId.length < 3) {
      showStatusMessage(statusMessage, 'User ID must be at least 3 characters long', 'error');
      return;
    }

    try {
      const result = await invoke('set_user_id', { userId });
      console.log('User ID set successfully:', result);
      showStatusMessage(statusMessage, 'User ID set successfully! Loading...', 'success');

      // Wait a bit before checking the status again
      setTimeout(async () => {
        // Check directly if user ID is set
        try {
          const isSet = await invoke('is_user_id_set');
          if (isSet) {
            showMainApp();
          } else {
            // If still not set, wait a bit more and try again
            setTimeout(() => {
              checkUserId();
            }, 300);
          }
        } catch (error) {
          console.error('Error checking user ID after setting:', error);
          showStatusMessage(statusMessage, `Error: ${error}`, 'error');
        }
      }, 800); // Increased timeout to ensure state is updated
    } catch (error) {
      console.error('Error setting user ID:', error);
      showStatusMessage(statusMessage, `Error: ${error}`, 'error');
    }
  });

  // Allow pressing Enter key to continue
  userIdInput.addEventListener('keypress', (event) => {
    if (event.key === 'Enter') {
      continueBtn.click();
    }
  });
}

// Function to show status message
function showStatusMessage(element: HTMLElement, message: string, type: string) {
  element.textContent = message;
  element.className = `status-message ${type}`;
  element.style.display = 'block';
  element.style.backgroundColor = type === 'error' ? 'rgba(244, 67, 54, 0.3)' : 'rgba(76, 175, 80, 0.3)';
  element.style.border = `1px solid ${type === 'error' ? '#f44336' : '#4CAF50'}`;
}

// Function to show main application UI
function showMainApp() {
  // If we have stored the original HTML, restore it
  if (originalHTML) {
    document.body.innerHTML = '';
    document.body.insertAdjacentHTML('afterbegin', '<div id="app-root"></div>');
    const appRoot = document.getElementById('app-root');
    if (appRoot) {
      appRoot.innerHTML = originalHTML;
    }
  }
  
  // Initialize main application components
  initializeMainAppComponents();
}

// Function to initialize main application components after user ID is set
function initializeMainAppComponents() {
  // Reinitialize all the original functionality from the main.ts file
  let recordBtn: HTMLButtonElement | null = document.getElementById("record-btn") as HTMLButtonElement;
  let stopBtn: HTMLButtonElement | null = document.getElementById("stop-btn") as HTMLButtonElement;
  let screenshotStatus: HTMLElement | null = document.getElementById("screenshot-status");
  let activityBadge: HTMLElement | null = document.getElementById("activity-badge");

  // Initialize network usage component (with global network stats)
  const networkUsage = new NetworkUsageComponent(true);
  networkUsage.start();

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

  // Add event listeners for buttons
  if (recordBtn) {
    recordBtn.onclick = startCombinedRecording;
  }
  if (stopBtn) {
    stopBtn.onclick = stopCombinedRecording;
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

  async function createAdminWindow() {
    try {
      // Call Rust function to create admin window
      await invoke("create_admin_window");
    } catch (error) {
      console.error("Error creating admin window:", error);
    }
  }
}

// Function to save a file to XAMPP htdocs directory
async function saveFileToXamppHtdocs(fileData: Uint8Array, filename: string, fileType: string): Promise<string> {
  try {
    const result = await invoke("save_file_to_xampp_htdocs", {
      fileData: Array.from(fileData),
      filename: filename,
      fileType: fileType
    });
    return result as string;
  } catch (error) {
    console.error(`Error saving ${fileType} to XAMPP htdocs:`, error);
    throw error;
  }
}

// Function to start system-wide idle monitoring in the backend
async function startSystemIdleMonitoring(): Promise<void> {
  try {
    const result = await invoke("start_system_idle_monitoring");
    console.log(result);
  } catch (error) {
    console.error('Error starting system idle monitoring:', error);
  }
}

// Function to stop system-wide idle monitoring in the backend
async function stopSystemIdleMonitoring(): Promise<void> {
  try {
    const result = await invoke("stop_system_idle_monitoring");
    console.log(result);
  } catch (error) {
    console.error('Error stopping system idle monitoring:', error);
  }
}

// Function to fetch and update the cached idle status directly
async function updateCachedIdleStatus(): Promise<void> {
  try {
    const cachedStatus = await invoke("get_cached_idle_status");
    updateIdleUI(cachedStatus as string);
  } catch (error) {
    console.error('Error fetching cached idle status:', error);
  }
}

// Function to update UI based on idle status
function updateIdleUI(status: string): void {
  const activityBadge = document.getElementById("activity-badge");
  if (activityBadge) {
    if (status === 'active') {
      activityBadge.style.backgroundColor = '#4CAF50'; // Green color for active
      activityBadge.style.boxShadow = '0 0 10px rgba(76, 175, 80, 0.7)'; // Green glow for active
    } else {
      activityBadge.style.backgroundColor = '#FFC107'; // Yellow color for idle
      activityBadge.style.boxShadow = '0 0 10px rgba(255, 193, 7, 0.7)'; // Yellow glow for idle
    }
  }
}

// Store the original HTML when the page first loads and run initial check
document.addEventListener('DOMContentLoaded', async () => {
  // Store the original HTML content structure
  originalHTML = document.body.innerHTML;
  await checkUserId();

  // Start monitoring system-wide idle status globally in the backend
  await startSystemIdleMonitoring();

  // Listen for system idle status updates from the backend
  try {
    await listen("system-idle-status", (event) => {
      const idleStatus = event.payload as { status: string, idleTimeSeconds: number };
      console.log(`System idle status: ${idleStatus.status}, idle time: ${idleStatus.idleTimeSeconds} seconds`);
      updateIdleUI(idleStatus.status);
    });
  } catch (error) {
    console.error('Error setting up idle status listener:', error);
  }

  // Listen for monitoring commands from system tray
  try {
    await listen("start-monitoring-request", () => {
      startSystemIdleMonitoring();
      console.log("Monitoring started via system tray");
    });

    await listen("stop-monitoring-request", () => {
      stopSystemIdleMonitoring();
      console.log("Monitoring stopped via system tray");
    });
  } catch (error) {
    console.error('Error setting up monitoring command listeners:', error);
  }

  // Set up periodic check of cached status (every 3 seconds) to handle throttling
  setInterval(async () => {
    await updateCachedIdleStatus();
  }, 3000);  // Check every 3 seconds
});

// Add keyboard shortcuts for window control
document.addEventListener('keydown', (event) => {
  // Ctrl+H to hide the window to system tray
  if (event.ctrlKey && event.key === 'h') {
    event.preventDefault();
    if (window.__TAURI__) {
      window.__TAURI__.webviewWindow.getCurrent().hide();
    }
  }

  // Ctrl+Shift+H to show the window
  if (event.ctrlKey && event.shiftKey && event.key === 'H') {
    event.preventDefault();
    if (window.__TAURI__) {
      window.__TAURI__.webviewWindow.getCurrent().show();
      window.__TAURI__.webviewWindow.getCurrent().setFocus();
    }
  }
});