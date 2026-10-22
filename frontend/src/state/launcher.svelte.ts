import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { 
  Tab, 
  Instance, 
  Account, 
  JavaInstallation, 
  GpuDevice, 
  LauncherSettings, 
  ModItem, 
  DownloadProgress, 
  GameExecutionState, 
  LauncherInfo 
} from "../types";

export interface ModrinthSearchResult {
  project_id: string;
  title: string;
  description: string;
  author: string;
  icon_url: string | null;
  downloads: number;
  follows: number;
  categories: string[];
}

export class LauncherState {
  activeTab = $state<Tab>("play");
  launcherInfo = $state<LauncherInfo>({ name: "Digital Vigital", version: "1.2.1", author: "Digital Vigital", repo_url: "" });
  
  // Profiles
  instances = $state<Instance[]>([]);
  activeInstance = $state<Instance | null>(null);
  
  // Accounts
  accounts = $state<Account[]>([]);
  activeAccount = $state<Account | null>(null);
  showAccountModal = $state(false);
  msLoginUrl = $state("");
  isMsLoggingIn = $state(false);
  
  // Modals
  showCreateInstanceModal = $state(false);
  
  // Game execution state
  gameStatus = $state<GameExecutionState>("idle");
  launchStatusText = $state("Ready to launch");
  downloadProgress = $state<DownloadProgress>({ current: 0, total: 0, percent: 0, status: "" });
  consoleLogs = $state<string[]>([]);
  autoScrollLogs = $state(true);

  // Hardware & Java state
  detectedJavas = $state<JavaInstallation[]>([]);
  detectedGpus = $state<GpuDevice[]>([]);
  isInstallingJava = $state(false);
  javaInstallStatus = $state("");
  javaInstallPercent = $state(0);

  // Settings
  settings = $state<LauncherSettings>({
    default_min_memory_mb: 1024,
    default_max_memory_mb: 4096,
    default_window_width: 1280,
    default_window_height: 720,
    selected_gpu: "auto",
    custom_jvm_args: "",
    performance_preset: "aikar",
    on_launch_action: "hide",
    enable_discord_rpc: true,
    data_dir: "",
    instances_dir: "",
    custom_java_path: null,
  });

  // Mods
  installedMods = $state<ModItem[]>([]);
  modrinthSearchResults = $state<ModrinthSearchResult[]>([]);
  isSearchingModrinth = $state(false);
  installingModId = $state("");
  
  // Notifications toast
  toastMessage = $state("");
  toastTimeout: any = null;

  showToast(msg: string) {
    this.toastMessage = msg;
    if (this.toastTimeout) clearTimeout(this.toastTimeout);
    this.toastTimeout = setTimeout(() => {
      this.toastMessage = "";
    }, 3500);
  }

  async init() {
    try {
      this.launcherInfo = await invoke("get_launcher_info");
    } catch (_) {}

    await this.refreshAccounts();
    await this.refreshInstances();
    await this.checkHardwareAndJava();
    await this.loadSettings();

    // Setup Tauri real-time event subscriptions
    try {
      await listen<string>("game-log", (event) => {
        this.consoleLogs.push(event.payload);
        if (this.consoleLogs.length > 2500) {
          this.consoleLogs = this.consoleLogs.slice(this.consoleLogs.length - 1500);
        }
      });

      await listen<string>("launch-status", (event) => {
        this.launchStatusText = event.payload;
      });

      await listen<any>("download-progress", (event) => {
        this.gameStatus = "downloading";
        this.downloadProgress = event.payload;
        this.launchStatusText = `Downloading: ${event.payload.status || "assets"} (${event.payload.percent.toFixed(1)}%)`;
      });

      await listen<string>("game-state-change", (event) => {
        const st = event.payload;
        if (st === "running") {
          this.gameStatus = "running";
          this.launchStatusText = "Minecraft is currently running";
        } else if (st === "starting") {
          this.gameStatus = "launching";
          this.launchStatusText = "Starting Minecraft JVM...";
        } else if (st === "finished" || st === "stopped") {
          this.gameStatus = "idle";
          this.launchStatusText = "Game process closed.";
        } else if (st === "crashed") {
          this.gameStatus = "idle";
          this.launchStatusText = "Game crashed. Check Live Console.";
          this.showToast("Minecraft exited unexpectedly.");
        }
      });

      await listen<any>("java-install-progress", (event) => {
        this.javaInstallStatus = event.payload.status;
        this.javaInstallPercent = event.payload.percent;
      });
    } catch (e) {
      console.warn("Event listeners registration error:", e);
    }

    try {
      const st: string = await invoke("get_game_status");
      if (st === "running") this.gameStatus = "running";
      else if (st === "launching") this.gameStatus = "launching";
    } catch (_) {}
  }

  // --- Instance Management ---
  async refreshInstances() {
    try {
      this.instances = await invoke("get_instances");
      this.activeInstance = this.instances.find(i => i.is_active) || null;
      if (this.activeInstance) {
        await this.loadInstalledMods();
      }
    } catch (e) {
      console.error("Failed to load instances:", e);
    }
  }

  async selectInstance(id: string) {
    try {
      await invoke("select_active_instance", { id });
      await this.refreshInstances();
      this.showToast("Profile selected!");
    } catch (e) {
      this.showToast("Error selecting profile: " + e);
    }
  }

  async createInstance(name: string, mcVersion: string, loader: string) {
    try {
      await invoke("create_instance", {
        name,
        mcVersion,
        loader
      });
      await this.refreshInstances();
      this.showToast(`Profile "${name}" created!`);
      this.showCreateInstanceModal = false;
    } catch (e) {
      this.showToast("Failed to create profile: " + e);
    }
  }

  async deleteInstance(id: string) {
    if (!confirm("Are you sure you want to delete this profile? All worlds and data will be permanently removed.")) return;
    try {
      await invoke("delete_instance", { id });
      if (this.activeInstance?.id === id) this.activeInstance = null;
      await this.refreshInstances();
      this.showToast("Profile deleted.");
    } catch (e) {
      this.showToast("Delete failed: " + e);
    }
  }

  // --- Account Management ---
  async refreshAccounts() {
    try {
      this.accounts = await invoke("get_accounts");
      this.activeAccount = this.accounts.find(a => a.is_active) || (this.accounts.length > 0 ? this.accounts[0] : null);
    } catch (e) {
      console.error("Failed to load accounts:", e);
    }
  }

  async selectAccount(id: string) {
    try {
      await invoke("select_active_account", { id });
      await this.refreshAccounts();
      this.showToast("Player account active!");
    } catch (e) {
      this.showToast("Account error: " + e);
    }
  }

  async addOfflineAccount(username: string) {
    const trimmed = username.trim();
    if (!trimmed) return;
    try {
      await invoke("add_offline_account", { username: trimmed });
      await this.refreshAccounts();
      this.showToast(`Welcome, ${trimmed}!`);
    } catch (e) {
      this.showToast("Failed to add account: " + e);
    }
  }

  async logoutAccount(id: string) {
    try {
      await invoke("logout_account", { id });
      await this.refreshAccounts();
      this.showToast("Account removed.");
    } catch (e) {
      this.showToast("Logout failed: " + e);
    }
  }

  async startMicrosoftLogin() {
    this.isMsLoggingIn = true;
    try {
      this.msLoginUrl = await invoke("start_microsoft_login");
      this.showToast("Browser opened! Authenticate with Microsoft & paste the code.");
    } catch (e) {
      this.showToast("Failed to open login: " + e);
      this.isMsLoggingIn = false;
    }
  }

  async completeMicrosoftLogin(code: string) {
    const trimmed = code.trim();
    if (!trimmed) return;
    try {
      await invoke("complete_microsoft_login", { code: trimmed });
      this.isMsLoggingIn = false;
      this.msLoginUrl = "";
      await this.refreshAccounts();
      this.showToast("Microsoft account connected!");
    } catch (e) {
      this.showToast("Authentication failed: " + e);
    }
  }

  // --- Mod Management ---
  async loadInstalledMods() {
    if (!this.activeInstance) return;
    try {
      this.installedMods = await invoke("get_installed_mods", { instanceId: this.activeInstance.id });
    } catch (e) {
      this.installedMods = [];
    }
  }

  async toggleMod(mod: ModItem) {
    if (!this.activeInstance) return;
    try {
      await invoke("toggle_mod", {
        instanceId: this.activeInstance.id,
        filename: mod.filename,
        enabled: !mod.is_enabled,
      });
      await this.loadInstalledMods();
    } catch (e) {
      this.showToast("Failed to toggle mod: " + e);
    }
  }

  async searchModrinth(query: string) {
    if (!query.trim() || !this.activeInstance) return;
    this.isSearchingModrinth = true;
    try {
      this.modrinthSearchResults = await invoke("search_modrinth", {
        query: query.trim(),
        loader: this.activeInstance.mod_loader.toLowerCase(),
        gameVersion: this.activeInstance.mc_version
      });
    } catch (e) {
      this.showToast("Mod search failed: " + e);
    } finally {
      this.isSearchingModrinth = false;
    }
  }

  async installModrinthMod(projectId: string) {
    if (!this.activeInstance) return;
    this.installingModId = projectId;
    try {
      const filename: string = await invoke("install_modrinth_mod", {
        instanceId: this.activeInstance.id,
        projectId
      });
      this.showToast(`Installed ${filename}!`);
      await this.loadInstalledMods();
    } catch (e) {
      this.showToast("Installation error: " + e);
    } finally {
      this.installingModId = "";
    }
  }

  async installEssentialFpsPack() {
    if (!this.activeInstance) {
      this.showToast("Select a profile first.");
      return;
    }
    const isFabric = this.activeInstance.mod_loader.toLowerCase().includes("fabric");
    const isForge = this.activeInstance.mod_loader.toLowerCase().includes("forge");
    
    this.showToast("Installing Essential Fast Performance Pack...");
    
    // Core FPS optimization mod slugs on Modrinth
    const performanceMods = isFabric 
      ? ["sodium", "lithium", "ferrite-core", "entityculling"]
      : isForge 
      ? ["embeddium", "ferrite-core", "entityculling"]
      : [];

    if (performanceMods.length === 0) {
      this.showToast("Essential FPS Pack requires a Fabric or Forge/NeoForge profile.");
      return;
    }

    let installedCount = 0;
    for (const modId of performanceMods) {
      try {
        this.installingModId = modId;
        await invoke("install_modrinth_mod", {
          instanceId: this.activeInstance.id,
          projectId: modId
        });
        installedCount++;
      } catch (err) {
        console.warn(`Could not install ${modId}:`, err);
      }
    }
    this.installingModId = "";
    await this.loadInstalledMods();
    this.showToast(`Installed ${installedCount} FPS optimization mods! Instant boost ready.`);
  }

  async openModsFolder() {
    if (!this.activeInstance) return;
    try {
      await invoke("open_mods_folder", { instanceId: this.activeInstance.id });
    } catch (e) {
      this.showToast("Failed to open folder: " + e);
    }
  }

  // --- Skins & Capes ---
  async pickAndApplySkin() {
    if (!this.activeAccount) {
      this.showToast("Select a player account first.");
      return;
    }
    try {
      const picked: string | null = await invoke("pick_skin_file");
      if (picked) {
        await invoke("apply_skin", { username: this.activeAccount.username, filePath: picked });
        this.showToast("Skin applied to your player profile!");
      }
    } catch (e) {
      this.showToast("Skin upload error: " + e);
    }
  }

  async pickAndApplyCape() {
    if (!this.activeAccount) {
      this.showToast("Select a player account first.");
      return;
    }
    try {
      const picked: string | null = await invoke("pick_cape_file");
      if (picked) {
        await invoke("apply_cape", { username: this.activeAccount.username, filePath: picked });
        this.showToast("Cape applied to your profile!");
      }
    } catch (e) {
      this.showToast("Cape upload error: " + e);
    }
  }

  // --- Hardware & Java ---
  async checkHardwareAndJava() {
    try {
      this.detectedJavas = await invoke("detect_java");
    } catch (_) {}
    try {
      this.detectedGpus = await invoke("detect_gpus");
    } catch (_) {}
  }

  async loadSettings() {
    try {
      this.settings = await invoke("get_settings");
    } catch (_) {}
  }

  async saveSettings() {
    try {
      await invoke("save_settings", { settings: this.settings });
      this.showToast("Settings saved successfully!");
    } catch (e) {
      this.showToast("Failed to save settings: " + e);
    }
  }

  async pickCustomJava() {
    try {
      const res: JavaInstallation | null = await invoke("pick_custom_java");
      if (res) {
        this.settings.custom_java_path = res.path;
        await this.saveSettings();
        await this.checkHardwareAndJava();
        this.showToast(`Selected Java: ${res.display_name}`);
      }
    } catch (e) {
      this.showToast("Failed to pick Java: " + e);
    }
  }

  async installJava(version: number) {
    this.isInstallingJava = true;
    this.javaInstallPercent = 0;
    this.javaInstallStatus = `Downloading Temurin OpenJDK ${version}...`;
    try {
      const path: string = await invoke("install_java", { majorVersion: version });
      this.showToast(`Java ${version} installed!`);
      this.javaInstallStatus = `Installed to ${path}`;
      await this.checkHardwareAndJava();
    } catch (e) {
      this.showToast("Java install error: " + e);
    } finally {
      setTimeout(() => {
        this.isInstallingJava = false;
        this.javaInstallStatus = "";
      }, 3500);
    }
  }

  // --- Launch & Terminate ---
  async launchOrStop() {
    if (this.gameStatus === "running") {
      try {
        await invoke("stop_game");
        this.showToast("Stopping Minecraft process...");
      } catch (e) {
        this.showToast("Failed to stop: " + e);
      }
      return;
    }

    if (!this.activeInstance) {
      this.showToast("Please choose or create a profile first!");
      this.activeTab = "instances";
      return;
    }

    this.gameStatus = "launching";
    this.launchStatusText = "Initializing launch sequence...";

    try {
      await invoke("launch_game", { instanceId: this.activeInstance.id });
    } catch (e: any) {
      this.gameStatus = "idle";
      this.launchStatusText = `Launch failed: ${e}`;
      this.showToast(`Error: ${e}`);
    }
  }

  async openFolder(path: string) {
    if (!path) return;
    try {
      await invoke("open_folder", { path });
    } catch (e) {
      this.showToast("Failed to open directory: " + e);
    }
  }
}

export const launcher = new LauncherState();
