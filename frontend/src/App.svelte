<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { 
    Gamepad2, 
    FolderKanban, 
    Wrench, 
    User, 
    Settings, 
    Terminal, 
    Play, 
    Square,
    Plus, 
    Trash2, 
    FolderOpen, 
    Cpu, 
    Coffee, 
    ShieldCheck, 
    RefreshCw, 
    Search, 
    Download, 
    Check, 
    Sparkles,
    ExternalLink,
    ChevronDown,
    LogOut,
    CheckCircle2,
    AlertCircle,
    Info,
    X,
    HardDrive,
    Sliders
  } from "lucide-svelte";
  import * as skinview3d from "skinview3d";

  // Navigation state
  type Tab = "play" | "instances" | "mods" | "skins" | "settings" | "console";
  let activeTab: Tab = $state("play");

  // App & Version state
  let launcherInfo = $state({ name: "Digital Vigital Launcher", version: "1.2.1", repo_url: "" });

  // Instances state
  interface Instance {
    id: string;
    name: string;
    mc_version: string;
    mod_loader: string;
    last_played: number | null;
    min_memory_mb: number;
    max_memory_mb: number;
    is_active: boolean;
  }
  let instances: Instance[] = $state([]);
  let activeInstance: Instance | null = $state(null);

  // Accounts state
  interface Account {
    id: string;
    username: string;
    account_type: "microsoft" | "offline" | string;
    avatar_url: string | null;
    last_login: number;
    is_active: boolean;
  }
  let accounts: Account[] = $state([]);
  let activeAccount: Account | null = $state(null);
  let showAccountsModal = $state(false);
  let newOfflineUsername = $state("");
  let msLoginUrl = $state("");
  let msAuthCode = $state("");
  let isMsLoggingIn = $state(false);

  // Game execution state
  let gameStatus: "idle" | "launching" | "downloading" | "running" = $state("idle");
  let launchStatusText = $state("Ready to launch");
  let downloadProgress = $state({ current: 0, total: 0, percent: 0, status: "", bytes: 0 });
  let consoleLogs: string[] = $state([]);
  let autoScrollLogs = $state(true);
  let consoleContainer: HTMLDivElement | null = $state(null);

  // Hardware & Java state
  interface JavaInstallation {
    path: string;
    vendor: string;
    version: string;
    major_version: number;
    display_name: string;
    is_recommended: boolean;
  }
  let detectedJavas: JavaInstallation[] = $state([]);
  let isInstallingJava = $state(false);
  let javaInstallStatus = $state("");
  let javaInstallPercent = $state(0);

  interface GpuDevice {
    id: string;
    name: string;
    is_discrete: boolean;
  }
  let detectedGpus: GpuDevice[] = $state([]);

  // Settings state
  let settingsState = $state({
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
    custom_java_path: ""
  });
  let settingsSaveSuccess = $state(false);

  // Mod Studio state
  interface ModItem {
    filename: string;
    clean_name: string;
    is_enabled: boolean;
    size: number;
  }
  let installedMods: ModItem[] = $state([]);
  let modrinthQuery = $state("");
  let modrinthResults: any[] = $state([]);
  let isSearchingModrinth = $state(false);
  let installingModId = $state("");

  // Skin Canvas state
  let skinCanvas: HTMLCanvasElement | null = $state(null);
  let skinViewer: skinview3d.SkinViewer | null = $state(null);
  let skinStatusMsg = $state("");

  // New Instance Modal state
  let showNewInstanceModal = $state(false);
  let newInstanceName = $state("");
  let newInstanceVersion = $state("1.21.4");
  let newInstanceLoader = $state("Fabric");

  // Notifications toast
  let toastMsg = $state("");
  function showToast(msg: string) {
    toastMsg = msg;
    setTimeout(() => { toastMsg = ""; }, 3500);
  }

  onMount(async () => {
    // 1. Load basic app metadata
    try {
      launcherInfo = await invoke("get_launcher_info");
    } catch (e) {
      console.warn("Launcher info fallback:", e);
    }

    // 2. Load accounts and instances
    await refreshAccounts();
    await refreshInstances();
    await checkJavaAndHardware();
    await loadSettings();

    // 3. Setup Tauri event listeners
    try {
      await listen<string>("game-log", (event) => {
        consoleLogs.push(event.payload);
        if (consoleLogs.length > 2000) {
          consoleLogs = consoleLogs.slice(consoleLogs.length - 1500);
        }
        if (autoScrollLogs && consoleContainer) {
          setTimeout(() => {
            if (consoleContainer) consoleContainer.scrollTop = consoleContainer.scrollHeight;
          }, 20);
        }
      });

      await listen<string>("launch-status", (event) => {
        launchStatusText = event.payload;
      });

      await listen<any>("download-progress", (event) => {
        gameStatus = "downloading";
        downloadProgress = event.payload;
        launchStatusText = `Downloading: ${event.payload.status || "assets"} (${event.payload.percent.toFixed(1)}%)`;
      });

      await listen<string>("game-state-change", (event) => {
        const st = event.payload;
        if (st === "running") {
          gameStatus = "running";
          launchStatusText = "Minecraft is currently running";
        } else if (st === "starting") {
          gameStatus = "launching";
          launchStatusText = "Starting Minecraft JVM...";
        } else if (st === "finished" || st === "stopped") {
          gameStatus = "idle";
          launchStatusText = "Game process closed.";
        } else if (st === "crashed") {
          gameStatus = "idle";
          launchStatusText = "Game crashed. Check the Console tab for details.";
          showToast("Minecraft exited unexpectedly. View Console logs.");
        }
      });

      await listen<any>("java-install-progress", (event) => {
        javaInstallStatus = event.payload.status;
        javaInstallPercent = event.payload.percent;
      });
    } catch (e) {
      console.warn("Event listeners registration error:", e);
    }

    // 4. Check initial game status
    try {
      const st: string = await invoke("get_game_status");
      if (st === "running") gameStatus = "running";
      else if (st === "launching") gameStatus = "launching";
    } catch (_) {}
  });

  // -------------------------------------------------------------------------
  // Subsystem Refresh Functions
  // -------------------------------------------------------------------------

  async function refreshInstances() {
    try {
      instances = await invoke("get_instances");
      activeInstance = instances.find((i) => i.is_active) || null;
      if (activeInstance) {
        await loadInstalledMods();
      }
    } catch (e) {
      console.error("Failed to load instances:", e);
    }
  }

  async function handleSelectInstance(id: string) {
    try {
      await invoke("select_active_instance", { id });
      await refreshInstances();
      showToast("Profile selected!");
    } catch (e) {
      showToast("Error selecting profile: " + e);
    }
  }

  async function refreshAccounts() {
    try {
      accounts = await invoke("get_accounts");
      activeAccount = accounts.find((a) => a.is_active) || (accounts.length > 0 ? accounts[0] : null);
    } catch (e) {
      console.error("Failed to load accounts:", e);
    }
  }

  async function handleSelectAccount(id: string) {
    try {
      await invoke("select_active_account", { id });
      await refreshAccounts();
      showToast("Switched player account!");
    } catch (e) {
      showToast("Error switching account: " + e);
    }
  }

  async function handleAddOfflineAccount() {
    const name = newOfflineUsername.trim();
    if (!name) return;
    try {
      await invoke("add_offline_account", { username: name });
      newOfflineUsername = "";
      await refreshAccounts();
      showToast(`Welcome, ${name}!`);
    } catch (e) {
      showToast("Account error: " + e);
    }
  }

  async function handleLogoutAccount(id: string) {
    try {
      await invoke("logout_account", { id });
      await refreshAccounts();
      showToast("Account removed.");
    } catch (e) {
      showToast("Logout failed: " + e);
    }
  }

  async function handleStartMicrosoftLogin() {
    isMsLoggingIn = true;
    try {
      msLoginUrl = await invoke("start_microsoft_login");
      showToast("Browser opened! Authenticate with Microsoft and paste the code below.");
    } catch (e) {
      showToast("Failed to open login: " + e);
      isMsLoggingIn = false;
    }
  }

  async function handleCompleteMicrosoftLogin() {
    if (!msAuthCode.trim()) return;
    try {
      await invoke("complete_microsoft_login", { code: msAuthCode.trim() });
      msAuthCode = "";
      isMsLoggingIn = false;
      await refreshAccounts();
      showToast("Microsoft account connected!");
    } catch (e) {
      showToast("Authentication failed: " + e);
    }
  }

  async function checkJavaAndHardware() {
    try {
      detectedJavas = await invoke("detect_java");
    } catch (e) {
      console.warn("Java scan error:", e);
    }
    try {
      detectedGpus = await invoke("detect_gpus");
    } catch (e) {
      console.warn("GPU scan error:", e);
    }
  }

  async function loadSettings() {
    try {
      settingsState = await invoke("get_settings");
    } catch (e) {
      console.warn("Load settings error:", e);
    }
  }

  async function handleSaveSettings() {
    try {
      await invoke("save_settings", { settings: settingsState });
      settingsSaveSuccess = true;
      showToast("Settings saved successfully!");
      setTimeout(() => { settingsSaveSuccess = false; }, 3000);
    } catch (e) {
      showToast("Failed to save settings: " + e);
    }
  }

  async function handleInstallJava(version: number) {
    isInstallingJava = true;
    javaInstallPercent = 0;
    javaInstallStatus = `Initializing Temurin OpenJDK ${version} download...`;
    try {
      const path = await invoke("install_java", { majorVersion: version });
      javaInstallStatus = `Successfully installed to ${path}`;
      showToast(`Java ${version} successfully installed!`);
      await checkJavaAndHardware();
    } catch (e: any) {
      javaInstallStatus = `Install failed: ${e}`;
      showToast(`Java install failed: ${e}`);
    } finally {
      setTimeout(() => {
        isInstallingJava = false;
        javaInstallStatus = "";
      }, 4000);
    }
  }

  async function handlePickCustomJava() {
    try {
      const res: JavaInstallation | null = await invoke("pick_custom_java");
      if (res) {
        settingsState.custom_java_path = res.path;
        await handleSaveSettings();
        await checkJavaAndHardware();
        showToast(`Selected Java: ${res.display_name}`);
      }
    } catch (e) {
      showToast("Failed to pick Java: " + e);
    }
  }

  // -------------------------------------------------------------------------
  // Instance CRUD Functions
  // -------------------------------------------------------------------------

  async function handleCreateInstance() {
    if (!newInstanceName.trim()) {
      showToast("Please enter an instance name");
      return;
    }
    try {
      await invoke("create_instance", {
        name: newInstanceName.trim(),
        mcVersion: newInstanceVersion,
        loader: newInstanceLoader
      });
      showNewInstanceModal = false;
      newInstanceName = "";
      await refreshInstances();
      showToast("Minecraft instance created!");
    } catch (e) {
      showToast("Failed to create instance: " + e);
    }
  }

  async function handleDeleteInstance(id: string) {
    if (!confirm("Are you sure you want to delete this profile? All worlds and mods inside will be erased.")) return;
    try {
      await invoke("delete_instance", { id });
      if (activeInstance?.id === id) activeInstance = null;
      await refreshInstances();
      showToast("Instance deleted.");
    } catch (e) {
      showToast("Failed to delete instance: " + e);
    }
  }

  // -------------------------------------------------------------------------
  // Mod Studio Functions
  // -------------------------------------------------------------------------

  async function loadInstalledMods() {
    if (!activeInstance) return;
    try {
      installedMods = await invoke("get_installed_mods", { instanceId: activeInstance.id });
    } catch (e) {
      console.warn("Failed to load mods:", e);
      installedMods = [];
    }
  }

  async function toggleMod(mod: ModItem) {
    if (!activeInstance) return;
    try {
      await invoke("toggle_mod", {
        instanceId: activeInstance.id,
        filename: mod.filename,
        enabled: !mod.is_enabled
      });
      await loadInstalledMods();
    } catch (e) {
      showToast("Failed to toggle mod: " + e);
    }
  }

  async function searchModrinth() {
    if (!modrinthQuery.trim() || !activeInstance) return;
    isSearchingModrinth = true;
    try {
      modrinthResults = await invoke("search_modrinth", {
        query: modrinthQuery.trim(),
        loader: activeInstance.mod_loader.toLowerCase(),
        gameVersion: activeInstance.mc_version
      });
    } catch (e) {
      showToast("Mod search failed: " + e);
    } finally {
      isSearchingModrinth = false;
    }
  }

  async function handleInstallModrinthMod(projectId: string) {
    if (!activeInstance) return;
    installingModId = projectId;
    try {
      const filename: string = await invoke("install_modrinth_mod", {
        instanceId: activeInstance.id,
        projectId
      });
      showToast(`Installed ${filename}!`);
      await loadInstalledMods();
    } catch (e) {
      showToast("Installation error: " + e);
    } finally {
      installingModId = "";
    }
  }

  async function handleOpenModsFolder() {
    if (!activeInstance) return;
    try {
      await invoke("open_mods_folder", { instanceId: activeInstance.id });
    } catch (e) {
      showToast("Failed to open folder: " + e);
    }
  }

  // -------------------------------------------------------------------------
  // 3D Skins & Capes Functions
  // -------------------------------------------------------------------------

  function initSkinViewer() {
    if (!skinCanvas) return;
    if (skinViewer) {
      skinViewer.dispose();
    }
    const avatarSkin = activeAccount?.id && activeAccount.account_type === "microsoft"
      ? `https://crafthead.net/skin/${activeAccount.id}`
      : "https://textures.minecraft.net/texture/3b60a1f6d2719b0d24e1644781df5a4628c64ee32f6b864f16a0daeaad6ad9e";

    skinViewer = new skinview3d.SkinViewer({
      canvas: skinCanvas,
      width: 320,
      height: 420,
      skin: avatarSkin
    });
    skinViewer.animation = new skinview3d.WalkingAnimation();
    skinViewer.animation.speed = 0.8;
  }

  $effect(() => {
    if (activeTab === "skins") {
      setTimeout(initSkinViewer, 50);
    }
  });

  async function handlePickAndApplySkin() {
    if (!activeAccount) {
      showToast("Select a player account first.");
      return;
    }
    try {
      const picked: string | null = await invoke("pick_skin_file");
      if (picked) {
        await invoke("apply_skin", { username: activeAccount.username, filePath: picked });
        showToast("Skin applied to your player profile!");
        initSkinViewer();
      }
    } catch (e) {
      showToast("Skin upload error: " + e);
    }
  }

  async function handlePickAndApplyCape() {
    if (!activeAccount) {
      showToast("Select a player account first.");
      return;
    }
    try {
      const picked: string | null = await invoke("pick_cape_file");
      if (picked) {
        await invoke("apply_cape", { username: activeAccount.username, filePath: picked });
        showToast("Cape applied to your profile!");
        initSkinViewer();
      }
    } catch (e) {
      showToast("Cape upload error: " + e);
    }
  }

  // -------------------------------------------------------------------------
  // Launch & Termination Functions
  // -------------------------------------------------------------------------

  async function handleLaunchOrStop() {
    if (gameStatus === "running") {
      try {
        await invoke("stop_game");
        showToast("Stopping Minecraft...");
      } catch (e) {
        showToast("Failed to stop game: " + e);
      }
      return;
    }

    if (!activeInstance) {
      showToast("Please choose or create a profile first!");
      activeTab = "instances";
      return;
    }

    gameStatus = "launching";
    launchStatusText = "Initializing launch sequence...";

    try {
      await invoke("launch_game", { instanceId: activeInstance.id });
    } catch (e: any) {
      gameStatus = "idle";
      launchStatusText = `Launch failed: ${e}`;
      showToast(`Error: ${e}`);
    }
  }

  async function handleOpenFolder(path: string) {
    if (!path) return;
    try {
      await invoke("open_folder", { path });
    } catch (e) {
      showToast("Failed to open directory: " + e);
    }
  }
</script>

<div class="flex h-screen w-screen overflow-hidden bg-[#0a0a0c] text-white select-none font-sans">
  <!-- ================= LEFT NAVIGATION SIDEBAR ================= -->
  <aside class="w-64 border-r border-white/6 bg-[#0e1017]/90 backdrop-blur-2xl flex flex-col justify-between p-4 shrink-0 shadow-2xl z-20">
    <div class="space-y-6">
      <!-- Brand Monogram Header -->
      <div class="flex items-center gap-3 px-2 pt-2">
        <div class="relative w-10 h-10 rounded-2xl bg-gradient-to-br from-red-500 via-rose-600 to-red-700 flex items-center justify-center shadow-lg shadow-red-500/30 group">
          <div class="font-mono font-black text-lg tracking-tighter text-white drop-shadow">DV</div>
          <span class="absolute -bottom-1 -right-1 w-3.5 h-3.5 rounded-full bg-emerald-500 border-2 border-[#0e1017]"></span>
        </div>
        <div>
          <h1 class="font-black text-sm tracking-tight uppercase font-mono text-white">Digital Vigital</h1>
          <div class="flex items-center gap-1.5 text-[10px] font-mono text-neutral-400">
            <span class="text-neutral-500">v{launcherInfo.version}</span>
            <span class="text-neutral-600">•</span>
            <span class="text-red-400 font-bold uppercase tracking-widest text-[9px]">TAURI v2</span>
          </div>
        </div>
      </div>

      <!-- Navigation Tabs -->
      <nav class="space-y-1">
        <button
          onclick={() => activeTab = "play"}
          class="w-full flex items-center gap-3 px-3 py-2.5 rounded-xl text-xs font-semibold uppercase tracking-wider transition-all cursor-pointer {activeTab === 'play' ? 'bg-red-500/15 text-red-400 border border-red-500/30 shadow-lg shadow-red-500/10' : 'text-neutral-400 hover:text-white hover:bg-white/4'}"
        >
          <Gamepad2 class="w-4 h-4" />
          <span>Launch & Play</span>
        </button>

        <button
          onclick={() => activeTab = "instances"}
          class="w-full flex items-center gap-3 px-3 py-2.5 rounded-xl text-xs font-semibold uppercase tracking-wider transition-all cursor-pointer {activeTab === 'instances' ? 'bg-red-500/15 text-red-400 border border-red-500/30 shadow-lg shadow-red-500/10' : 'text-neutral-400 hover:text-white hover:bg-white/4'}"
        >
          <FolderKanban class="w-4 h-4" />
          <span>Profiles ({instances.length})</span>
        </button>

        <button
          onclick={() => activeTab = "mods"}
          class="w-full flex items-center gap-3 px-3 py-2.5 rounded-xl text-xs font-semibold uppercase tracking-wider transition-all cursor-pointer {activeTab === 'mods' ? 'bg-red-500/15 text-red-400 border border-red-500/30 shadow-lg shadow-red-500/10' : 'text-neutral-400 hover:text-white hover:bg-white/4'}"
        >
          <Wrench class="w-4 h-4" />
          <span>Mod Studio</span>
        </button>

        <button
          onclick={() => activeTab = "skins"}
          class="w-full flex items-center gap-3 px-3 py-2.5 rounded-xl text-xs font-semibold uppercase tracking-wider transition-all cursor-pointer {activeTab === 'skins' ? 'bg-red-500/15 text-red-400 border border-red-500/30 shadow-lg shadow-red-500/10' : 'text-neutral-400 hover:text-white hover:bg-white/4'}"
        >
          <User class="w-4 h-4" />
          <span>3D Skins & Capes</span>
        </button>

        <button
          onclick={() => activeTab = "settings"}
          class="w-full flex items-center gap-3 px-3 py-2.5 rounded-xl text-xs font-semibold uppercase tracking-wider transition-all cursor-pointer {activeTab === 'settings' ? 'bg-red-500/15 text-red-400 border border-red-500/30 shadow-lg shadow-red-500/10' : 'text-neutral-400 hover:text-white hover:bg-white/4'}"
        >
          <Settings class="w-4 h-4" />
          <span>Java & Engine</span>
        </button>

        <button
          onclick={() => activeTab = "console"}
          class="w-full flex items-center gap-3 px-3 py-2.5 rounded-xl text-xs font-semibold uppercase tracking-wider transition-all cursor-pointer {activeTab === 'console' ? 'bg-red-500/15 text-red-400 border border-red-500/30 shadow-lg shadow-red-500/10' : 'text-neutral-400 hover:text-white hover:bg-white/4'}"
        >
          <Terminal class="w-4 h-4" />
          <span>Live Console</span>
        </button>
      </nav>
    </div>

    <!-- Quick Status Hardware Footer -->
    <div class="p-3 rounded-2xl bg-white/3 border border-white/6 space-y-2 text-[11px] font-mono text-neutral-400">
      <div class="flex items-center justify-between">
        <span class="flex items-center gap-1.5"><Coffee class="w-3.5 h-3.5 text-amber-400" /> Java Runtime</span>
        <span class="text-neutral-200 font-bold truncate max-w-[100px]">
          {detectedJavas.find(j => j.is_recommended)?.display_name || (detectedJavas[0]?.display_name || "Auto")}
        </span>
      </div>
      <div class="flex items-center justify-between">
        <span class="flex items-center gap-1.5"><Cpu class="w-3.5 h-3.5 text-sky-400" /> GPU Engine</span>
        <span class="text-neutral-200 font-bold truncate max-w-[100px]">
          {detectedGpus.find(g => g.is_discrete)?.name.replace(/NVIDIA|GeForce|AMD|Radeon/gi, "").trim() || "Dedicated"}
        </span>
      </div>
      <div class="flex items-center justify-between pt-1 border-t border-white/6 text-[10px]">
        <span class="text-neutral-500">Status</span>
        {#if gameStatus === "running"}
          <span class="text-emerald-400 font-bold flex items-center gap-1">
            <span class="w-1.5 h-1.5 rounded-full bg-emerald-400 animate-pulse"></span>
            Running
          </span>
        {:else if gameStatus === "launching" || gameStatus === "downloading"}
          <span class="text-amber-400 font-bold flex items-center gap-1">
            <span class="w-1.5 h-1.5 rounded-full bg-amber-400 animate-spin"></span>
            Starting
          </span>
        {:else}
          <span class="text-neutral-400">Standby</span>
        {/if}
      </div>
    </div>
  </aside>

  <!-- ================= MAIN CONTENT VIEWPORT ================= -->
  <main class="flex-1 flex flex-col overflow-hidden bg-[#0c0d12]">
    <!-- Top Global App Bar -->
    <header class="h-16 border-b border-white/6 px-8 flex items-center justify-between bg-[#10121a]/60 backdrop-blur-xl shrink-0 z-10">
      <!-- Active Instance Quick Selector -->
      <div class="flex items-center gap-3">
        <span class="text-[11px] font-mono uppercase tracking-widest text-neutral-400">Profile:</span>
        {#if activeInstance}
          <div class="px-3.5 py-1.5 rounded-xl bg-white/4 border border-white/8 font-mono text-xs font-bold text-white flex items-center gap-2.5">
            <span class="w-2 h-2 rounded-full bg-red-500"></span>
            <span>{activeInstance.name}</span>
            <span class="text-[10px] px-1.5 py-0.2 rounded bg-white/6 text-neutral-300 font-normal">
              {activeInstance.mod_loader} • {activeInstance.mc_version}
            </span>
          </div>
        {:else}
          <button
            onclick={() => activeTab = "instances"}
            class="px-3.5 py-1.5 rounded-xl bg-red-500/10 border border-red-500/30 font-mono text-xs font-bold text-red-400 flex items-center gap-2 hover:bg-red-500/20 transition-all cursor-pointer"
          >
            <AlertCircle class="w-3.5 h-3.5" />
            <span>Choose a Profile</span>
          </button>
        {/if}
      </div>

      <!-- Player Account Profile Widget -->
      <button
        onclick={() => showAccountsModal = true}
        class="flex items-center gap-3 px-3 py-1.5 rounded-2xl bg-white/4 hover:bg-white/8 border border-white/8 transition-all cursor-pointer group"
      >
        <div class="relative w-7 h-7 rounded-lg overflow-hidden bg-neutral-800 flex items-center justify-center font-bold text-xs">
          {#if activeAccount?.avatar_url}
            <img src={activeAccount.avatar_url} alt="" class="w-full h-full object-cover" />
          {:else}
            <span class="text-neutral-400 font-mono">{activeAccount?.username?.slice(0, 2).toUpperCase() || "DV"}</span>
          {/if}
        </div>
        <div class="text-left font-mono">
          <div class="text-xs font-bold text-white group-hover:text-red-400 transition-colors">
            {activeAccount?.username || "Guest Player"}
          </div>
          <div class="text-[9px] uppercase tracking-wider">
            {#if activeAccount?.account_type === "microsoft"}
              <span class="text-sky-400">Microsoft MSA</span>
            {:else}
              <span class="text-emerald-400">Offline / Dev</span>
            {/if}
          </div>
        </div>
        <ChevronDown class="w-3.5 h-3.5 text-neutral-500 group-hover:text-white transition-colors" />
      </button>
    </header>

    <!-- Scrollable Tab Content -->
    <div class="flex-1 overflow-y-auto p-8">
      {#if activeTab === "play"}
        <!-- ================= PLAY DASHBOARD TAB ================= -->
        <div class="max-w-4xl mx-auto space-y-8">
          {#if !activeInstance}
            <!-- Empty State: No Profile Selected -->
            <div class="p-10 rounded-3xl border border-white/10 bg-[#141622]/60 text-center space-y-5">
              <div class="w-16 h-16 mx-auto rounded-2xl bg-red-500/10 border border-red-500/20 flex items-center justify-center text-red-400">
                <FolderKanban class="w-8 h-8" />
              </div>
              <div class="space-y-1.5">
                <h3 class="text-xl font-bold font-mono text-white uppercase">No Minecraft Profile Selected</h3>
                <p class="text-xs text-neutral-400 max-w-md mx-auto">
                  To launch the game, select an existing profile or create a fresh isolated instance with your favorite version and mod loader.
                </p>
              </div>
              <div class="flex items-center justify-center gap-3 pt-2">
                <button
                  onclick={() => showNewInstanceModal = true}
                  class="px-5 py-2.5 rounded-xl bg-red-600 hover:bg-red-500 text-white font-mono text-xs font-bold uppercase tracking-wider transition-all cursor-pointer shadow-lg shadow-red-500/25"
                >
                  Create New Instance
                </button>
                <button
                  onclick={() => activeTab = "instances"}
                  class="px-5 py-2.5 rounded-xl bg-white/6 hover:bg-white/10 border border-white/10 text-white font-mono text-xs font-bold uppercase tracking-wider transition-all cursor-pointer"
                >
                  Browse Profiles
                </button>
              </div>
            </div>
          {:else}
            <!-- Hero Launch Card -->
            <div class="relative overflow-hidden rounded-3xl border border-white/10 bg-gradient-to-br from-[#161824] via-[#12131c] to-[#0c0d12] p-8 sm:p-10 shadow-2xl">
              <div class="absolute -top-24 -right-24 w-80 h-80 bg-red-500/12 blur-[100px] rounded-full pointer-events-none"></div>

              <div class="relative z-10 space-y-6">
                <div class="inline-flex items-center gap-2 px-3 py-1 rounded-full bg-red-500/10 border border-red-500/30 text-red-400 text-xs font-mono font-bold uppercase tracking-widest">
                  <Sparkles class="w-3.5 h-3.5" />
                  <span>Native High-FPS Pipeline</span>
                </div>

                <div class="space-y-2">
                  <h2 class="text-3xl sm:text-4xl font-black uppercase tracking-tight text-white font-mono">
                    {activeInstance.name}
                  </h2>
                  <p class="text-xs sm:text-sm text-neutral-400 max-w-xl leading-relaxed">
                    Powered by pure Rust process management, automatic discrete GPU acceleration, and privacy-first credential vaulting.
                  </p>
                </div>

                <!-- Specs Grid -->
                <div class="grid grid-cols-3 gap-4 pt-2">
                  <div class="p-4 rounded-2xl bg-white/3 border border-white/8 space-y-1 font-mono text-xs">
                    <div class="text-neutral-500 uppercase text-[10px] tracking-wider">Mod Loader</div>
                    <div class="font-bold text-neutral-100">{activeInstance.mod_loader} {activeInstance.mc_version}</div>
                  </div>
                  <div class="p-4 rounded-2xl bg-white/3 border border-white/8 space-y-1 font-mono text-xs">
                    <div class="text-neutral-500 uppercase text-[10px] tracking-wider">Memory Limit</div>
                    <div class="font-bold text-red-400">{activeInstance.max_memory_mb} MB RAM</div>
                  </div>
                  <div class="p-4 rounded-2xl bg-white/3 border border-white/8 space-y-1 font-mono text-xs">
                    <div class="text-neutral-500 uppercase text-[10px] tracking-wider">Installed Mods</div>
                    <div class="font-bold text-neutral-100">{installedMods.length} Optimizations</div>
                  </div>
                </div>

                <!-- Download Progress Indicator (if active) -->
                {#if gameStatus === "downloading"}
                  <div class="p-4 rounded-2xl bg-white/4 border border-white/10 space-y-2 font-mono text-xs">
                    <div class="flex items-center justify-between text-neutral-300">
                      <span class="truncate max-w-md">{downloadProgress.status || "Downloading game assets..."}</span>
                      <span class="font-bold text-red-400">{downloadProgress.percent.toFixed(1)}%</span>
                    </div>
                    <div class="w-full h-2 rounded-full bg-white/10 overflow-hidden">
                      <div class="h-full bg-gradient-to-r from-red-600 to-rose-500 transition-all duration-200" style="width: {downloadProgress.percent}%"></div>
                    </div>
                  </div>
                {/if}

                <!-- Main Action Button -->
                <div class="pt-2">
                  <button
                    onclick={handleLaunchOrStop}
                    disabled={gameStatus === "launching"}
                    class="w-full flex items-center justify-center gap-3 py-4 rounded-2xl font-black text-sm uppercase tracking-widest transition-all duration-300 shadow-xl cursor-pointer disabled:opacity-50 {gameStatus === 'running' ? 'bg-gradient-to-r from-neutral-800 to-neutral-900 border border-red-500/40 text-red-400 hover:bg-red-500/20' : 'bg-gradient-to-r from-red-600 via-rose-600 to-red-500 hover:from-red-500 hover:to-rose-500 text-white shadow-red-500/25 hover:shadow-red-500/40 hover:scale-[1.008] active:scale-[0.995]'}"
                  >
                    {#if gameStatus === "running"}
                      <Square class="w-5 h-5 fill-current" />
                      <span>Terminate Minecraft Process</span>
                    {:else if gameStatus === "launching" || gameStatus === "downloading"}
                      <RefreshCw class="w-5 h-5 animate-spin" />
                      <span>{launchStatusText}</span>
                    {:else}
                      <Play class="w-5 h-5 fill-current" />
                      <span>Launch Minecraft</span>
                    {/if}
                  </button>
                </div>
              </div>
            </div>

            <!-- Official Community SMP Banner -->
            <div class="p-6 rounded-3xl border border-white/8 bg-[#141620]/60 flex items-center justify-between shadow-xl">
              <div class="space-y-1">
                <div class="flex items-center gap-2">
                  <span class="w-2 h-2 rounded-full bg-emerald-400 animate-ping"></span>
                  <span class="text-[11px] font-mono font-bold uppercase tracking-wider text-emerald-400">Official Community Realm</span>
                </div>
                <h4 class="text-base font-bold text-white">Digital Vigital SMP (Paper 1.21+)</h4>
                <p class="text-xs text-neutral-400">Exclusive Discord whitelist, custom seasonal events, and ultra-smooth tick rate.</p>
              </div>
              <a
                href="https://discord.gg/flex-dv"
                target="_blank"
                class="px-4 py-2.5 rounded-xl bg-white/6 hover:bg-white/10 border border-white/12 text-xs font-mono font-bold uppercase tracking-wider transition-all flex items-center gap-2 text-white"
              >
                <span>Join Whitelist</span>
                <ExternalLink class="w-3.5 h-3.5" />
              </a>
            </div>
          {/if}
        </div>

      {:else if activeTab === "instances"}
        <!-- ================= INSTANCES TAB ================= -->
        <div class="max-w-4xl mx-auto space-y-6">
          <div class="flex items-center justify-between">
            <div>
              <h2 class="text-xl font-bold uppercase tracking-tight font-mono text-white">Profiles & Instances</h2>
              <p class="text-xs text-neutral-400">Manage isolated Minecraft installations, versions, and mod loader configurations.</p>
            </div>
            <button
              onclick={() => showNewInstanceModal = true}
              class="flex items-center gap-2 px-4 py-2 rounded-xl bg-red-600 hover:bg-red-500 text-white font-mono text-xs font-bold uppercase tracking-wider transition-all cursor-pointer shadow-lg shadow-red-500/25"
            >
              <Plus class="w-4 h-4" />
              <span>New Instance</span>
            </button>
          </div>

          <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
            {#each instances as inst}
              <div class="p-5 rounded-2xl border {activeInstance?.id === inst.id ? 'border-red-500/50 bg-red-500/5' : 'border-white/8 bg-[#141620]/60'} space-y-4 shadow-lg transition-all">
                <div class="flex items-start justify-between">
                  <div class="space-y-1">
                    <h4 class="font-bold text-sm text-white">{inst.name}</h4>
                    <span class="text-xs font-mono text-neutral-400">{inst.mod_loader} • MC {inst.mc_version}</span>
                  </div>
                  {#if activeInstance?.id === inst.id}
                    <span class="px-2 py-0.5 rounded bg-red-500/20 text-red-400 text-[10px] font-mono font-bold uppercase">Active</span>
                  {/if}
                </div>

                <div class="flex items-center justify-between text-[11px] font-mono text-neutral-400 pt-2 border-t border-white/6">
                  <span>RAM: {inst.max_memory_mb} MB</span>
                  <span>{inst.last_played ? new Date(inst.last_played * 1000).toLocaleDateString() : "Never played"}</span>
                </div>

                <div class="flex items-center gap-2 pt-2">
                  <button
                    onclick={() => handleSelectInstance(inst.id)}
                    class="flex-1 py-1.5 rounded-lg {activeInstance?.id === inst.id ? 'bg-red-500/20 text-red-300 font-bold' : 'bg-white/6 hover:bg-white/10 text-neutral-200'} text-xs font-mono uppercase tracking-wider transition-all cursor-pointer"
                  >
                    {activeInstance?.id === inst.id ? "Selected" : "Select Profile"}
                  </button>
                  <button
                    onclick={() => handleDeleteInstance(inst.id)}
                    class="p-2 rounded-lg bg-white/4 hover:bg-red-500/20 text-neutral-400 hover:text-red-400 transition-all cursor-pointer"
                    title="Delete Instance"
                  >
                    <Trash2 class="w-3.5 h-3.5" />
                  </button>
                </div>
              </div>
            {/each}
          </div>
        </div>

      {:else if activeTab === "mods"}
        <!-- ================= MOD STUDIO TAB ================= -->
        <div class="max-w-4xl mx-auto space-y-6">
          <div class="flex items-center justify-between">
            <div>
              <h2 class="text-xl font-bold uppercase tracking-tight font-mono text-white">Mod Studio</h2>
              <p class="text-xs text-neutral-400">Discover and 1-click install mods from Modrinth or manage local profile jars.</p>
            </div>
            {#if activeInstance}
              <button
                onclick={handleOpenModsFolder}
                class="flex items-center gap-2 px-3.5 py-2 rounded-xl bg-white/6 hover:bg-white/10 border border-white/10 text-white font-mono text-xs font-bold uppercase tracking-wider transition-all cursor-pointer"
              >
                <FolderOpen class="w-3.5 h-3.5" />
                <span>Open Mods Folder</span>
              </button>
            {/if}
          </div>

          <!-- Mod Search Bar -->
          <div class="flex items-center gap-3">
            <div class="flex-1 relative">
              <Search class="w-4 h-4 text-neutral-400 absolute left-3.5 top-1/2 -translate-y-1/2" />
              <input
                type="text"
                bind:value={modrinthQuery}
                onkeydown={(e) => e.key === 'Enter' && searchModrinth()}
                placeholder="Search Modrinth (e.g. Sodium, Iris, Zoomify, Fabric API, Lithium)..."
                class="w-full pl-10 pr-4 py-2.5 rounded-xl bg-white/6 border border-white/10 text-xs font-mono text-white placeholder-neutral-500 focus:outline-none focus:border-red-500"
              />
            </div>
            <button
              onclick={searchModrinth}
              disabled={isSearchingModrinth}
              class="px-5 py-2.5 rounded-xl bg-red-600 hover:bg-red-500 text-white font-mono text-xs font-bold uppercase tracking-wider transition-all cursor-pointer disabled:opacity-50"
            >
              {isSearchingModrinth ? "Searching..." : "Search"}
            </button>
          </div>

          <!-- Modrinth Search Results -->
          {#if modrinthResults.length > 0}
            <div class="space-y-3">
              <h3 class="text-xs font-mono uppercase tracking-widest text-neutral-400">Search Results from Modrinth</h3>
              <div class="grid grid-cols-1 gap-2.5">
                {#each modrinthResults as mod}
                  <div class="p-4 rounded-xl bg-[#141620]/80 border border-white/8 flex items-center justify-between">
                    <div class="space-y-1 max-w-xl">
                      <div class="flex items-center gap-2">
                        <span class="text-xs font-bold text-white">{mod.title}</span>
                        <span class="text-[10px] font-mono text-neutral-400">by {mod.author}</span>
                      </div>
                      <p class="text-[11px] text-neutral-400 line-clamp-1">{mod.description}</p>
                    </div>
                    <button
                      onclick={() => handleInstallModrinthMod(mod.project_id || mod.slug)}
                      disabled={installingModId === (mod.project_id || mod.slug)}
                      class="px-3.5 py-1.5 rounded-lg bg-red-600 hover:bg-red-500 text-white font-mono text-xs font-bold uppercase tracking-wider transition-all cursor-pointer disabled:opacity-50"
                    >
                      {installingModId === (mod.project_id || mod.slug) ? "Installing..." : "Install"}
                    </button>
                  </div>
                {/each}
              </div>
            </div>
          {/if}

          <!-- Installed Mods List -->
          <div class="space-y-3">
            <h3 class="text-xs font-mono uppercase tracking-widest text-neutral-400">Installed Mods ({installedMods.length})</h3>
            {#if installedMods.length === 0}
              <div class="p-8 rounded-2xl border border-white/6 bg-white/2 text-center text-xs font-mono text-neutral-500">
                No mods installed in this profile. Search Modrinth above or drop .jar files into the mods folder.
              </div>
            {:else}
              <div class="space-y-2">
                {#each installedMods as mod}
                  <div class="p-3.5 rounded-xl bg-[#141620]/60 border border-white/8 flex items-center justify-between">
                    <div class="space-y-0.5">
                      <div class="text-xs font-bold text-white">{mod.clean_name}</div>
                      <div class="text-[10px] font-mono text-neutral-500">{mod.filename} • {(mod.size / 1024).toFixed(0)} KB</div>
                    </div>
                    <button
                      onclick={() => toggleMod(mod)}
                      class="px-3 py-1 rounded-lg text-xs font-mono font-bold uppercase transition-all cursor-pointer {mod.is_enabled ? 'bg-emerald-500/20 text-emerald-400 border border-emerald-500/30' : 'bg-neutral-800 text-neutral-500'}"
                    >
                      {mod.is_enabled ? "Enabled" : "Disabled"}
                    </button>
                  </div>
                {/each}
              </div>
            {/if}
          </div>
        </div>

      {:else if activeTab === "skins"}
        <!-- ================= 3D SKINS & CAPES TAB ================= -->
        <div class="max-w-4xl mx-auto space-y-6">
          <div>
            <h2 class="text-xl font-bold uppercase tracking-tight font-mono text-white">3D Skin & Cape Studio</h2>
            <p class="text-xs text-neutral-400">Interactive live 3D preview rendered via WebGL with native universal injection.</p>
          </div>

          <div class="grid grid-cols-12 gap-8 items-center">
            <!-- 3D Canvas Box -->
            <div class="col-span-5 p-6 rounded-3xl border border-white/8 bg-[#141620]/80 flex flex-col items-center justify-center shadow-2xl">
              <canvas bind:this={skinCanvas} class="rounded-xl"></canvas>
            </div>

            <!-- Controls & Uploads -->
            <div class="col-span-7 space-y-5 font-mono text-xs">
              <div class="p-5 rounded-2xl bg-white/3 border border-white/8 space-y-2">
                <h4 class="font-bold text-white uppercase text-sm">Universal Texture Injector</h4>
                <p class="text-neutral-400 text-[11px] leading-relaxed">
                  Your custom skin and cape are automatically synchronized across all profiles, Digital Vigital SMP realms, and singleplayer worlds without third-party mod dependencies.
                </p>
              </div>

              <div class="space-y-3">
                <button
                  onclick={handlePickAndApplySkin}
                  class="w-full py-3 px-4 rounded-xl bg-white/6 hover:bg-white/10 border border-white/10 text-white font-bold uppercase tracking-wider transition-all flex items-center justify-center gap-2 cursor-pointer"
                >
                  <User class="w-4 h-4 text-red-400" />
                  <span>Upload Custom Skin (.png)</span>
                </button>
                <button
                  onclick={handlePickAndApplyCape}
                  class="w-full py-3 px-4 rounded-xl bg-white/6 hover:bg-white/10 border border-white/10 text-white font-bold uppercase tracking-wider transition-all flex items-center justify-center gap-2 cursor-pointer"
                >
                  <Sparkles class="w-4 h-4 text-sky-400" />
                  <span>Upload Custom Cape (.png)</span>
                </button>
              </div>
            </div>
          </div>
        </div>

      {:else if activeTab === "settings"}
        <!-- ================= SETTINGS & JAVA ENGINE TAB ================= -->
        <div class="max-w-4xl mx-auto space-y-6">
          <div class="flex items-center justify-between">
            <div>
              <h2 class="text-xl font-bold uppercase tracking-tight font-mono text-white">Settings & Java Engine</h2>
              <p class="text-xs text-neutral-400">Configure Java runtimes, dedicated GPU selection, GC presets, and memory limits.</p>
            </div>
            <button
              onclick={handleSaveSettings}
              class="px-5 py-2 rounded-xl bg-red-600 hover:bg-red-500 text-white font-mono text-xs font-bold uppercase tracking-wider transition-all cursor-pointer shadow-lg shadow-red-500/25"
            >
              Save Preferences
            </button>
          </div>

          <!-- Automatic Multi-Java Installer -->
          <div class="p-6 rounded-3xl border border-white/10 bg-[#161822] space-y-4">
            <div class="flex items-center justify-between">
              <div>
                <h4 class="text-sm font-bold text-white flex items-center gap-2">
                  <Coffee class="w-4 h-4 text-amber-400" />
                  <span>1-Click Java Runtime Manager</span>
                </h4>
                <p class="text-xs text-neutral-400">Official Eclipse Adoptium Temurin OpenJDK builds for Minecraft compatibility.</p>
              </div>
              <div class="flex gap-2">
                <button
                  onclick={() => handleInstallJava(21)}
                  disabled={isInstallingJava}
                  class="px-3.5 py-1.5 rounded-xl bg-red-600 hover:bg-red-500 text-white font-mono text-xs font-bold uppercase tracking-wider transition-all cursor-pointer disabled:opacity-50"
                >
                  Install Java 21
                </button>
                <button
                  onclick={() => handleInstallJava(17)}
                  disabled={isInstallingJava}
                  class="px-3.5 py-1.5 rounded-xl bg-white/6 hover:bg-white/10 border border-white/10 text-white font-mono text-xs font-bold uppercase tracking-wider transition-all cursor-pointer disabled:opacity-50"
                >
                  Install Java 17
                </button>
                <button
                  onclick={() => handleInstallJava(8)}
                  disabled={isInstallingJava}
                  class="px-3.5 py-1.5 rounded-xl bg-white/6 hover:bg-white/10 border border-white/10 text-white font-mono text-xs font-bold uppercase tracking-wider transition-all cursor-pointer disabled:opacity-50"
                >
                  Install Java 8
                </button>
              </div>
            </div>

            {#if javaInstallStatus}
              <div class="p-3 rounded-xl bg-white/4 border border-white/6 font-mono text-xs text-neutral-300 space-y-2">
                <div>{javaInstallStatus}</div>
                {#if isInstallingJava && javaInstallPercent > 0}
                  <div class="w-full h-1.5 rounded-full bg-white/10 overflow-hidden">
                    <div class="h-full bg-red-500 transition-all duration-200" style="width: {javaInstallPercent}%"></div>
                  </div>
                {/if}
              </div>
            {/if}

            <!-- Detected Java Runtimes -->
            <div class="pt-2 space-y-2">
              <div class="text-[11px] font-mono uppercase tracking-wider text-neutral-400">Detected System Runtimes ({detectedJavas.length})</div>
              <div class="space-y-1.5">
                {#each detectedJavas as j}
                  <div class="p-2.5 rounded-xl bg-white/3 border border-white/6 flex items-center justify-between font-mono text-xs">
                    <div class="truncate max-w-lg">
                      <span class="font-bold text-white">{j.display_name}</span>
                      <span class="text-neutral-500 ml-2 text-[10px] truncate">{j.path}</span>
                    </div>
                    {#if j.is_recommended}
                      <span class="px-2 py-0.5 rounded bg-emerald-500/20 text-emerald-400 text-[10px] font-bold">Recommended</span>
                    {/if}
                  </div>
                {/each}
              </div>
            </div>
          </div>

          <!-- Memory Allocation Slider -->
          <div class="p-6 rounded-3xl border border-white/8 bg-[#141620]/60 space-y-4 font-mono text-xs">
            <div class="flex justify-between">
              <span class="text-neutral-300 font-bold uppercase">Memory Allocation (RAM)</span>
              <span class="text-red-400 font-bold">{settingsState.default_max_memory_mb} MB ({(settingsState.default_max_memory_mb / 1024).toFixed(1)} GB)</span>
            </div>
            <input
              type="range"
              min="2048"
              max="16384"
              step="1024"
              bind:value={settingsState.default_max_memory_mb}
              class="w-full accent-red-500 cursor-pointer"
            />
          </div>

          <!-- GPU & JVM Preset -->
          <div class="grid grid-cols-2 gap-4 font-mono text-xs">
            <div class="p-6 rounded-3xl border border-white/8 bg-[#141620]/60 space-y-2">
              <label class="text-neutral-300 font-bold uppercase">Graphics Engine (GPU)</label>
              <select
                bind:value={settingsState.selected_gpu}
                class="w-full px-3 py-2 rounded-xl bg-neutral-900 border border-white/10 text-white focus:outline-none focus:border-red-500"
              >
                <option value="auto">Automatic (Dedicated Discrete GPU Preferred)</option>
                {#each detectedGpus as gpu}
                  <option value={gpu.id}>{gpu.name} {gpu.is_discrete ? "(Discrete)" : ""}</option>
                {/each}
              </select>
            </div>

            <div class="p-6 rounded-3xl border border-white/8 bg-[#141620]/60 space-y-2">
              <label class="text-neutral-300 font-bold uppercase">JVM Garbage Collector Preset</label>
              <select
                bind:value={settingsState.performance_preset}
                class="w-full px-3 py-2 rounded-xl bg-neutral-900 border border-white/10 text-white focus:outline-none focus:border-red-500"
              >
                <option value="aikar">Aikar's Flags (High FPS & Zero Stutter)</option>
                <option value="zgc">Generational ZGC (Java 21+ Modern)</option>
                <option value="shenandoah">Shenandoah GC (Ultra-Low Latency)</option>
                <option value="default">Default / Vanilla JVM</option>
              </select>
            </div>
          </div>
        </div>

      {:else if activeTab === "console"}
        <!-- ================= LIVE CONSOLE TAB ================= -->
        <div class="max-w-4xl mx-auto space-y-4 font-mono">
          <div class="flex items-center justify-between">
            <div class="flex items-center gap-2">
              <h2 class="text-xs uppercase tracking-widest text-neutral-400 font-bold">Process Streaming Output</h2>
              <span class="text-[10px] text-emerald-400 flex items-center gap-1.5">
                <span class="w-1.5 h-1.5 rounded-full bg-emerald-400 animate-ping"></span>
                ACTIVE
              </span>
            </div>
            <div class="flex items-center gap-3">
              <button
                onclick={() => autoScrollLogs = !autoScrollLogs}
                class="text-[11px] text-neutral-400 hover:text-white transition-colors cursor-pointer"
              >
                Auto-scroll: {autoScrollLogs ? "ON" : "OFF"}
              </button>
              <button
                onclick={() => consoleLogs = []}
                class="px-3 py-1 rounded-lg bg-white/6 hover:bg-white/10 text-[11px] text-neutral-300 transition-all cursor-pointer"
              >
                Clear Console
              </button>
            </div>
          </div>

          <div
            bind:this={consoleContainer}
            class="h-[520px] rounded-3xl bg-black/90 border border-white/8 p-5 font-mono text-xs text-neutral-300 overflow-y-auto space-y-1 shadow-2xl"
          >
            {#if consoleLogs.length === 0}
              <div class="text-neutral-500">[DV] Standing by for Minecraft game execution...</div>
            {:else}
              {#each consoleLogs as log}
                <div class="break-all leading-relaxed hover:bg-white/2">{log}</div>
              {/each}
            {/if}
          </div>
        </div>
      {/if}
    </div>
  </main>
</div>

<!-- ================= MODAL: ACCOUNTS MANAGEMENT ================= -->
{#if showAccountsModal}
  <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/80 backdrop-blur-md p-4">
    <div class="w-full max-w-md rounded-3xl border border-white/10 bg-[#141620] p-6 space-y-5 shadow-2xl">
      <div class="flex items-center justify-between">
        <h3 class="text-base font-bold text-white uppercase font-mono">Player Accounts</h3>
        <button onclick={() => showAccountsModal = false} class="p-1 rounded-lg text-neutral-400 hover:text-white cursor-pointer">
          <X class="w-4 h-4" />
        </button>
      </div>

      <!-- Accounts list -->
      <div class="space-y-2 max-h-56 overflow-y-auto">
        {#each accounts as acc}
          <div class="p-3 rounded-2xl bg-white/4 border {acc.is_active ? 'border-red-500/50 bg-red-500/5' : 'border-white/6'} flex items-center justify-between font-mono text-xs">
            <div class="flex items-center gap-2.5">
              <div class="w-7 h-7 rounded-lg overflow-hidden bg-neutral-800 flex items-center justify-center font-bold">
                {#if acc.avatar_url}
                  <img src={acc.avatar_url} alt="" class="w-full h-full object-cover" />
                {:else}
                  <span class="text-[10px] text-neutral-400">{acc.username.slice(0, 2).toUpperCase()}</span>
                {/if}
              </div>
              <div>
                <div class="font-bold text-white">{acc.username}</div>
                <div class="text-[9px] text-neutral-400 uppercase">{acc.account_type}</div>
              </div>
            </div>

            <div class="flex items-center gap-2">
              {#if !acc.is_active}
                <button
                  onclick={() => handleSelectAccount(acc.id)}
                  class="px-2.5 py-1 rounded-lg bg-white/6 hover:bg-white/10 text-neutral-200 text-[11px] cursor-pointer"
                >
                  Activate
                </button>
              {:else}
                <span class="text-red-400 font-bold text-[10px] uppercase">Active</span>
              {/if}
              <button
                onclick={() => handleLogoutAccount(acc.id)}
                class="p-1.5 rounded-lg hover:bg-red-500/20 text-neutral-400 hover:text-red-400 cursor-pointer"
                title="Remove"
              >
                <Trash2 class="w-3.5 h-3.5" />
              </button>
            </div>
          </div>
        {/each}
      </div>

      <!-- Add Offline Account Form -->
      <div class="p-4 rounded-2xl bg-white/3 border border-white/6 space-y-3 font-mono text-xs">
        <label class="text-neutral-400 font-bold uppercase">Add Offline Account</label>
        <div class="flex gap-2">
          <input
            type="text"
            bind:value={newOfflineUsername}
            placeholder="Username (e.g. Steve_606)"
            class="flex-1 px-3 py-2 rounded-xl bg-white/6 border border-white/10 text-white placeholder-neutral-500 focus:outline-none focus:border-red-500"
          />
          <button
            onclick={handleAddOfflineAccount}
            class="px-4 py-2 rounded-xl bg-red-600 hover:bg-red-500 text-white font-bold uppercase cursor-pointer"
          >
            Add
          </button>
        </div>
      </div>

      <!-- Microsoft Login Flow -->
      <div class="pt-2">
        {#if !isMsLoggingIn}
          <button
            onclick={handleStartMicrosoftLogin}
            class="w-full py-2.5 rounded-xl bg-sky-600 hover:bg-sky-500 text-white font-mono text-xs font-bold uppercase tracking-wider transition-all cursor-pointer shadow-lg shadow-sky-500/20"
          >
            Login with Microsoft (PKCE OAuth)
          </button>
        {:else}
          <div class="space-y-2 font-mono text-xs">
            <input
              type="text"
              bind:value={msAuthCode}
              placeholder="Paste Microsoft redirect code here..."
              class="w-full px-3 py-2 rounded-xl bg-white/6 border border-white/10 text-white placeholder-neutral-500 focus:outline-none focus:border-sky-500"
            />
            <button
              onclick={handleCompleteMicrosoftLogin}
              class="w-full py-2 rounded-xl bg-sky-600 hover:bg-sky-500 text-white font-bold uppercase cursor-pointer"
            >
              Verify Microsoft Code
            </button>
          </div>
        {/if}
      </div>
    </div>
  </div>
{/if}

<!-- ================= MODAL: CREATE NEW INSTANCE ================= -->
{#if showNewInstanceModal}
  <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/80 backdrop-blur-md p-4">
    <div class="w-full max-w-md rounded-3xl border border-white/10 bg-[#141620] p-6 space-y-5 shadow-2xl">
      <div class="flex items-center justify-between">
        <h3 class="text-base font-bold text-white uppercase font-mono">Create New Profile</h3>
        <button onclick={() => showNewInstanceModal = false} class="p-1 rounded-lg text-neutral-400 hover:text-white cursor-pointer">
          <X class="w-4 h-4" />
        </button>
      </div>

      <div class="space-y-4 font-mono text-xs">
        <div class="space-y-1.5">
          <label class="text-neutral-400">Profile Name</label>
          <input
            type="text"
            bind:value={newInstanceName}
            placeholder="e.g. Fabric 1.21.4 Performance"
            class="w-full px-3 py-2.5 rounded-xl bg-white/6 border border-white/10 text-white placeholder-neutral-500 focus:outline-none focus:border-red-500"
          />
        </div>

        <div class="space-y-1.5">
          <label class="text-neutral-400">Minecraft Version</label>
          <select
            bind:value={newInstanceVersion}
            class="w-full px-3 py-2.5 rounded-xl bg-neutral-900 border border-white/10 text-white focus:outline-none focus:border-red-500"
          >
            <option value="1.21.4">1.21.4 (Latest Release • Java 21)</option>
            <option value="1.20.1">1.20.1 (Major Modded • Java 17)</option>
            <option value="1.19.4">1.19.4 (Java 17)</option>
            <option value="1.16.5">1.16.5 (Legacy • Java 8)</option>
            <option value="1.12.2">1.12.2 (Classic Forge • Java 8)</option>
            <option value="1.8.9">1.8.9 (PvP Standard • Java 8)</option>
          </select>
        </div>

        <div class="space-y-1.5">
          <label class="text-neutral-400">Mod Loader</label>
          <select
            bind:value={newInstanceLoader}
            class="w-full px-3 py-2.5 rounded-xl bg-neutral-900 border border-white/10 text-white focus:outline-none focus:border-red-500"
          >
            <option value="Fabric">Fabric (Recommended for High FPS)</option>
            <option value="Vanilla">Vanilla</option>
            <option value="NeoForge">NeoForge</option>
            <option value="Forge">Forge</option>
          </select>
        </div>
      </div>

      <div class="flex gap-3 pt-2">
        <button
          onclick={() => showNewInstanceModal = false}
          class="flex-1 py-2.5 rounded-xl bg-white/6 hover:bg-white/10 text-xs font-mono font-bold uppercase text-neutral-300 cursor-pointer"
        >
          Cancel
        </button>
        <button
          onclick={handleCreateInstance}
          class="flex-1 py-2.5 rounded-xl bg-red-600 hover:bg-red-500 text-xs font-mono font-bold uppercase text-white shadow-lg shadow-red-500/25 cursor-pointer"
        >
          Create Profile
        </button>
      </div>
    </div>
  </div>
{/if}

<!-- ================= TOAST NOTIFICATION ================= -->
{#if toastMsg}
  <div class="fixed bottom-6 right-6 z-50 px-4 py-3 rounded-2xl bg-neutral-900/90 border border-white/15 text-white font-mono text-xs shadow-2xl flex items-center gap-3 backdrop-blur-xl animate-fade-in">
    <Info class="w-4 h-4 text-red-400 shrink-0" />
    <span>{toastMsg}</span>
  </div>
{/if}
