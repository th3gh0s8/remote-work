import { invoke } from "@tauri-apps/api/core";

export class NetworkUsageComponent {
  private downloadSpeedEl: HTMLElement | null = null;
  private uploadSpeedEl: HTMLElement | null = null;
  private totalDownloadedEl: HTMLElement | null = null;
  private totalUploadedEl: HTMLElement | null = null;
  private intervalId: number | null = null;
  private useGlobalNetwork: boolean;

  constructor(useGlobalNetwork: boolean = false) {
    // Initialize network monitoring elements
    this.downloadSpeedEl = document.querySelector("#download-speed");
    this.uploadSpeedEl = document.querySelector("#upload-speed");
    this.totalDownloadedEl = document.querySelector("#total-downloaded");
    this.totalUploadedEl = document.querySelector("#total-uploaded");

    // Determine whether to use global network stats or app-specific stats
    this.useGlobalNetwork = useGlobalNetwork;
  }

  // Function to update network usage display
  private async updateNetworkUsage() {
    try {
      let stats;
      if (this.useGlobalNetwork) {
        stats = await invoke('get_global_network_stats');
      } else {
        stats = await invoke('get_network_stats');
      }

      const statsObj = JSON.parse(stats as string);

      if (this.downloadSpeedEl) this.downloadSpeedEl.textContent = statsObj.downloadSpeed;
      if (this.uploadSpeedEl) this.uploadSpeedEl.textContent = statsObj.uploadSpeed;
      if (this.totalDownloadedEl) this.totalDownloadedEl.textContent = statsObj.totalDownloaded;
      if (this.totalUploadedEl) this.totalUploadedEl.textContent = statsObj.totalUploaded;
    } catch (error) {
      console.error("Error getting network usage:", error);
    }
  }

  // Start the network usage update interval
  public start() {
    // Update network usage periodically (every 2 seconds)
    this.updateNetworkUsage(); // Initial update
    this.intervalId = window.setInterval(() => {
      this.updateNetworkUsage();
    }, 2000);
  }

  // Stop the network usage update interval
  public stop() {
    if (this.intervalId) {
      clearInterval(this.intervalId);
      this.intervalId = null;
    }
  }
}