import { invoke } from "@tauri-apps/api/core";

export class NetworkStatsComponent {
  private downloadSpeedEl: HTMLElement | null = null;
  private uploadSpeedEl: HTMLElement | null = null;
  private totalDownloadedEl: HTMLElement | null = null;
  private totalUploadedEl: HTMLElement | null = null;
  private intervalId: number | null = null;

  constructor() {
    // Initialize network monitoring elements
    this.downloadSpeedEl = document.querySelector("#download-speed");
    this.uploadSpeedEl = document.querySelector("#upload-speed");
    this.totalDownloadedEl = document.querySelector("#total-downloaded");
    this.totalUploadedEl = document.querySelector("#total-uploaded");
  }

  // Function to update network statistics display
  private async updateNetworkStats() {
    try {
      const stats = await invoke('get_network_stats');
      const statsObj = JSON.parse(stats as string);

      if (this.downloadSpeedEl) this.downloadSpeedEl.textContent = statsObj.downloadSpeed;
      if (this.uploadSpeedEl) this.uploadSpeedEl.textContent = statsObj.uploadSpeed;
      if (this.totalDownloadedEl) this.totalDownloadedEl.textContent = statsObj.totalDownloaded;
      if (this.totalUploadedEl) this.totalUploadedEl.textContent = statsObj.totalUploaded;
    } catch (error) {
      console.error("Error getting network stats:", error);
    }
  }

  // Start the network stats update interval
  public start() {
    // Update network stats periodically (every 2 seconds)
    this.updateNetworkStats(); // Initial update
    this.intervalId = window.setInterval(() => {
      this.updateNetworkStats();
    }, 2000);
  }

  // Stop the network stats update interval
  public stop() {
    if (this.intervalId) {
      clearInterval(this.intervalId);
      this.intervalId = null;
    }
  }
}