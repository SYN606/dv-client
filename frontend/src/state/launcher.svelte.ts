import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { TauriService } from "../services/tauri";
import { DEFAULT_STEVE_SKIN_DATA_URL } from "../constants/skins";
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
  LauncherInfo,
  MinecraftVersionSummary,
  ModProvider,
  CatalogModItem,
  PlayerSkinPayload,
  UpdateCheckResult,
  CrashAnalysis,
} from "../types";

export interface ModrinthSearchResult {
  project_id: string;
  slug?: string;
  title: string;
  description: string;
  author: string;
  icon_url: string | null;
  downloads: number;
  follows?: number;
  categories: string[];
  versions?: string[];
}

export class LauncherState {
  activeTab = $state<Tab>("play");
  launcherInfo = $state<LauncherInfo>({ name: "Digital Vigital", version: "1.0.0", author: "Digital Vigital", repo_url: "" });

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

  // Game execution state & Logs
  gameStatus = $state<GameExecutionState>("idle");
  launchStatusText = $state("Ready to launch");
  downloadProgress = $state<DownloadProgress>({ current: 0, total: 0, percent: 0, status: "" });
  consoleLogs = $state<string[]>([]);
  autoScrollLogs = $state(true);
  logSearchQuery = $state("");
  logLevelFilter = $state<"ALL" | "INFO" | "WARN" | "ERROR">("ALL");
  isReportingCrash = $state(false);
  crashReportSuccess = $state(false);
  crashDetails = $state<CrashAnalysis | null>(null);
  hasManualCrash = $state(false);

  // Software Updates
  updateInfo = $state<UpdateCheckResult | null>(null);
  isCheckingUpdates = $state(false);
  showUpdateModal = $state(false);
  dismissedUpdateBanner = $state(false);

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

  // Minecraft Versions
  availableMinecraftVersions = $state<MinecraftVersionSummary[]>([]);
  isLoadingVersions = $state(false);

  // Mods
  installedMods = $state<ModItem[]>([]);
  modStudioSubTab = $state<"installed" | "catalog">("installed");
  installedModsSearch = $state("");
  installedModsFilter = $state<"all" | "enabled" | "disabled">("all");
  modProvider = $state<ModProvider>("modrinth");
  catalogMods = $state<CatalogModItem[]>([]);
  modrinthSearchResults = $state<ModrinthSearchResult[]>([]);
  isSearchingModrinth = $state(false);
  isSearchingCatalog = $state(false);
  installingModId = $state("");
  modrinthCategory = $state("all");
  modrinthSortBy = $state("downloads");
  modrinthQuery = $state("");

  // 3D Skin & Cape state
  activeSkinDataUrl = $state(DEFAULT_STEVE_SKIN_DATA_URL);
  activeCapeDataUrl = $state<string | null>(null);
  isLoadingSkin = $state(false);
  skinModelType = $state<"classic" | "slim">("classic");

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
    } catch (_) { }

    await this.refreshAccounts();
    await this.refreshInstances();
    await this.checkHardwareAndJava();
    await this.loadSettings();
    await this.loadMinecraftVersions();
    await this.loadPlayerSkin();

    // Check for launcher updates in background
    this.checkForUpdates(false).catch(() => {});

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
          this.gameStatus = "crashed";
          this.launchStatusText = "Game crashed. Check Live Console.";
          this.showToast("Minecraft exited unexpectedly.");
        }
      });

      await listen<CrashAnalysis>("game-crashed", (event) => {
        this.crashDetails = event.payload;
        this.hasManualCrash = true;
        this.gameStatus = "crashed";
        if (event.payload?.headline) {
          this.launchStatusText = `Crashed: ${event.payload.headline}`;
          this.showToast(`[${event.payload.category}] ${event.payload.headline}`);
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
    } catch (_) { }
  }

  // --- Instance Management ---
  async refreshInstances() {
    try {
      this.instances = await invoke("get_instances");
      this.activeInstance = this.instances.find(i => i.is_active) || null;
      if (this.activeInstance) {
        await this.loadInstalledMods();
        this.fetchModsForSelectedProfile();
      }
    } catch (e) {
      console.error("Failed to load instances:", e);
    }
  }

  async selectInstance(id: string) {
    try {
      await invoke("select_active_instance", { id });
      await this.refreshInstances();
      this.fetchModsForSelectedProfile();
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
      await this.loadPlayerSkin();
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

  // --- Version Management ---
  async loadMinecraftVersions() {
    this.isLoadingVersions = true;
    try {
      this.availableMinecraftVersions = await invoke("get_all_minecraft_versions");
    } catch (e) {
      console.warn("Failed to load official Minecraft versions:", e);
    } finally {
      this.isLoadingVersions = false;
    }
  }

  // --- Mod Management ---
  async loadInstalledMods() {
    if (!this.activeInstance) return;
    try {
      const rawMods: ModItem[] = await invoke("get_installed_mods", { instanceId: this.activeInstance.id });
      this.installedMods = (rawMods || []).map(m => ({
        ...m,
        file_name: m.file_name || m.filename || "mod.jar",
        name: m.name || m.clean_name || m.file_name || "Unknown Mod",
        file_size_display: m.file_size_display || `${((m.file_size_bytes || 0) / 1024).toFixed(1)} KB`,
      }));
    } catch (e) {
      this.installedMods = [];
    }
  }

  getInstalledModFor(mod: { id?: string; title: string; slug?: string }): ModItem | null {
    if (!this.installedMods || !this.installedMods.length) return null;
    const targetId = mod.id ? String(mod.id).trim().toLowerCase() : "";
    const targetSlug = mod.slug ? mod.slug.trim().toLowerCase() : "";
    const targetTitle = (mod.title || "").toLowerCase().trim();
    const cleanTargetTitle = targetTitle.replace(/[^a-z0-9]/g, "");

    return this.installedMods.find(m => {
      // 1. Strict Project UUID / ID match (Highest precision - Modrinth project ID or CurseForge ID)
      if (targetId && m.project_id) {
        if (String(m.project_id).trim().toLowerCase() === targetId) {
          return true;
        }
      }

      // 2. Strict Slug match
      if (targetSlug && m.slug) {
        if (m.slug.trim().toLowerCase() === targetSlug) {
          return true;
        }
      }

      // 3. Strict mod_id match (from fabric.mod.json / mods.toml)
      if (targetSlug && m.mod_id) {
        if (m.mod_id.trim().toLowerCase() === targetSlug) {
          return true;
        }
      }

      // 4. Exact Title Match (equality ONLY, never substring includes)
      const mNameClean = (m.name || "").toLowerCase().replace(/[^a-z0-9]/g, "");
      if (cleanTargetTitle && mNameClean && cleanTargetTitle === mNameClean) {
        return true;
      }

      // 5. Strict filename prefix match (e.g. "sodium-fabric-0.6.14.jar")
      // MUST NOT be a loose substring: only if followed by '-', '_', or '.'
      const mFileLower = (m.file_name || m.filename || "").toLowerCase();
      if (targetSlug && targetSlug.length >= 3) {
        if (
          mFileLower.startsWith(`${targetSlug}-`) ||
          mFileLower.startsWith(`${targetSlug}_`) ||
          mFileLower.startsWith(`${targetSlug}.`) ||
          mFileLower === `${targetSlug}.jar` ||
          mFileLower === `${targetSlug}.jar.disabled`
        ) {
          return true;
        }
      }

      return false;
    }) || null;
  }

  isModInstalled(mod: { id?: string; title: string; slug?: string }): boolean {
    return this.getInstalledModFor(mod) !== null;
  }

  async fetchModsForSelectedProfile(category?: string, sortBy?: string, query?: string) {
    if (!this.activeInstance) return;
    if (category !== undefined) this.modrinthCategory = category;
    if (sortBy !== undefined) this.modrinthSortBy = sortBy;
    if (query !== undefined) this.modrinthQuery = query;

    this.isSearchingCatalog = true;
    this.isSearchingModrinth = true;
    try {
      const isVanilla = this.activeInstance.mod_loader.toLowerCase() === "vanilla";
      const loaderParam = isVanilla ? null : this.activeInstance.mod_loader.toLowerCase();
      const q = this.modrinthQuery.trim();

      if (this.modProvider === "curseforge") {
        const results: any[] = await invoke("search_curseforge", {
          query: q || null,
          loader: loaderParam,
          gameVersion: this.activeInstance.mc_version,
          categoryId: null,
          sortBy: this.modrinthSortBy,
          limit: 36,
        });

        this.catalogMods = (results || []).map(cf => ({
          provider: "curseforge",
          id: cf.id.toString(),
          slug: cf.slug || cf.name.toLowerCase().replace(/[^a-z0-9]/g, "-"),
          title: cf.name,
          description: cf.summary,
          author: cf.author,
          icon_url: cf.icon_url,
          downloads: cf.download_count,
          categories: cf.categories || [],
          website_url: cf.website_url,
        }));
      } else {
        const catParam = this.modrinthCategory === "all" ? null : this.modrinthCategory;
        const results: ModrinthSearchResult[] = await invoke("search_modrinth", {
          query: q || null,
          loader: loaderParam,
          gameVersion: this.activeInstance.mc_version,
          category: catParam,
          sortBy: this.modrinthSortBy,
          limit: 36,
        });

        this.modrinthSearchResults = results || [];
        this.catalogMods = (results || []).map(mr => ({
          provider: "modrinth",
          id: mr.project_id,
          slug: mr.slug || mr.title.toLowerCase().replace(/[^a-z0-9]/g, "-"),
          title: mr.title,
          description: mr.description,
          author: mr.author,
          icon_url: mr.icon_url,
          downloads: mr.downloads,
          categories: mr.categories || [],
        }));
      }
    } catch (e) {
      console.warn("Failed to fetch mods from catalog:", e);
    } finally {
      this.isSearchingCatalog = false;
      this.isSearchingModrinth = false;
    }
  }

  async searchModrinth(query: string) {
    this.modrinthQuery = query;
    await this.fetchModsForSelectedProfile(undefined, undefined, query);
  }

  async toggleMod(mod: ModItem) {
    if (!this.activeInstance) return;
    const targetFile = mod.file_name || mod.filename;
    if (!targetFile) return;
    try {
      await invoke("toggle_mod", {
        instanceId: this.activeInstance.id,
        filename: targetFile,
        enabled: !mod.is_enabled,
      });
      await this.loadInstalledMods();
    } catch (e) {
      this.showToast("Failed to toggle mod: " + e);
    }
  }

  async bulkToggleMods(enableAll: boolean) {
    if (!this.activeInstance) return;
    try {
      const count: number = await invoke("bulk_toggle_mods", {
        instanceId: this.activeInstance.id,
        enableAll,
      });
      this.showToast(`${enableAll ? "Enabled" : "Disabled"} ${count} mods.`);
      await this.loadInstalledMods();
    } catch (e) {
      this.showToast("Failed to bulk update mods: " + e);
    }
  }

  async deleteMod(filename: string) {
    if (!this.activeInstance) return;
    try {
      await invoke("delete_mod", {
        instanceId: this.activeInstance.id,
        filename,
      });
      this.showToast(`Mod "${filename}" removed.`);
      await this.loadInstalledMods();
    } catch (e) {
      this.showToast("Failed to delete mod: " + e);
    }
  }

  async importLocalMod() {
    if (!this.activeInstance) {
      this.showToast("Select a profile first.");
      return;
    }
    try {
      const imported: string | null = await invoke("import_local_mod", {
        instanceId: this.activeInstance.id,
      });
      if (imported) {
        this.showToast(`Imported ${imported} into profile!`);
        await this.loadInstalledMods();
      }
    } catch (e) {
      this.showToast("Import failed: " + e);
    }
  }

  async installCatalogMod(mod: CatalogModItem) {
    if (!this.activeInstance) return;
    this.installingModId = mod.id;
    try {
      let filename = "";
      if (mod.provider === "curseforge") {
        filename = await invoke("install_curseforge_mod", {
          instanceId: this.activeInstance.id,
          modId: parseInt(mod.id, 10),
          slug: mod.slug,
          title: mod.title,
        });
      } else {
        filename = await invoke("install_modrinth_mod", {
          instanceId: this.activeInstance.id,
          projectId: mod.id,
          slug: mod.slug,
          title: mod.title,
        });
      }
      this.showToast(`Installed ${filename || mod.title}!`);
      await this.loadInstalledMods();
    } catch (e) {
      this.showToast("Installation error: " + e);
    } finally {
      this.installingModId = "";
    }
  }

  async uninstallCatalogMod(mod: { id?: string; title: string; slug?: string }) {
    if (!this.activeInstance) return;
    const installed = this.getInstalledModFor(mod);
    try {
      if (installed) {
        await invoke("delete_mod", {
          instanceId: this.activeInstance.id,
          filename: installed.file_name || installed.filename || "",
        });
      }
      const lookupSlug = mod.slug || mod.id;
      if (lookupSlug) {
        await invoke("uninstall_modrinth_mod", {
          instanceId: this.activeInstance.id,
          slug: lookupSlug,
        });
      }
      this.showToast(`Uninstalled ${mod.title}!`);
      await this.loadInstalledMods();
    } catch (e) {
      this.showToast("Uninstall error: " + e);
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
    const loaderLower = this.activeInstance.mod_loader.toLowerCase();
    const isFabricOrQuilt = loaderLower.includes("fabric") || loaderLower.includes("quilt");
    const isForgeOrNeo = loaderLower.includes("forge");

    if (loaderLower === "vanilla") {
      this.showToast("Vanilla profiles do not support mods. Create a Fabric or Forge profile.");
      return;
    }

    this.showToast("Installing Essential Fast Performance Pack...");

    // Core FPS optimization mod slugs on Modrinth
    const performanceMods = isFabricOrQuilt
      ? ["sodium", "lithium", "ferrite-core", "entityculling"]
      : isForgeOrNeo
        ? ["embeddium", "ferrite-core", "entityculling"]
        : [];

    if (performanceMods.length === 0) {
      this.showToast("Essential FPS Pack requires a Fabric, Quilt, Forge, or NeoForge profile.");
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
  async loadPlayerSkin(username?: string) {
    this.isLoadingSkin = true;
    try {
      const u = username || this.activeAccount?.username || null;
      const res: PlayerSkinPayload = await TauriService.getPlayerSkin(u);
      if (res && res.skin_data_url) {
        this.activeSkinDataUrl = res.skin_data_url;
      } else {
        this.activeSkinDataUrl = DEFAULT_STEVE_SKIN_DATA_URL;
      }
      this.activeCapeDataUrl = res?.cape_data_url || null;
    } catch (e) {
      this.activeSkinDataUrl = DEFAULT_STEVE_SKIN_DATA_URL;
      this.activeCapeDataUrl = null;
    } finally {
      this.isLoadingSkin = false;
    }
  }

  async pickAndApplySkin() {
    if (!this.activeAccount) {
      this.showToast("Select a player account first.");
      return;
    }
    try {
      const picked: string | null = await TauriService.pickSkinFile();
      if (picked) {
        const res: PlayerSkinPayload = await TauriService.applySkin(this.activeAccount.username, picked);
        if (res && res.skin_data_url) {
          this.activeSkinDataUrl = res.skin_data_url;
          this.activeCapeDataUrl = res.cape_data_url || null;
        }
        this.showToast("Skin applied to your player profile!");
      }
    } catch (e) {
      this.showToast("Skin upload error: " + e);
    }
  }

  async applySkinBase64(base64Data: string) {
    if (!this.activeAccount) {
      this.showToast("Select a player account first.");
      return;
    }
    try {
      this.isLoadingSkin = true;
      const res: PlayerSkinPayload = await TauriService.applySkinBase64(this.activeAccount.username, base64Data);
      if (res && res.skin_data_url) {
        this.activeSkinDataUrl = res.skin_data_url;
      }
      this.showToast("Skin applied to profile!");
    } catch (e: any) {
      this.showToast("Failed to apply skin: " + e);
    } finally {
      this.isLoadingSkin = false;
    }
  }

  async pickAndApplyCape() {
    if (!this.activeAccount) {
      this.showToast("Select a player account first.");
      return;
    }
    try {
      const picked: string | null = await TauriService.pickCapeFile();
      if (picked) {
        const res: PlayerSkinPayload = await TauriService.applyCape(this.activeAccount.username, picked);
        if (res && res.cape_data_url) {
          this.activeCapeDataUrl = res.cape_data_url;
        }
        this.showToast("Cape applied to your profile!");
      }
    } catch (e) {
      this.showToast("Cape upload error: " + e);
    }
  }

  async applyCapeBase64(base64Data: string) {
    if (!this.activeAccount) {
      this.showToast("Select a player account first.");
      return;
    }
    try {
      this.isLoadingSkin = true;
      const res: PlayerSkinPayload = await TauriService.applyCapeBase64(this.activeAccount.username, base64Data);
      if (res && res.cape_data_url) {
        this.activeCapeDataUrl = res.cape_data_url;
      }
      this.showToast("Cape applied to profile!");
    } catch (e: any) {
      this.showToast("Failed to apply cape: " + e);
    } finally {
      this.isLoadingSkin = false;
    }
  }

  async importSkinByUsername(username: string) {
    if (!this.activeAccount) {
      this.showToast("Select a player account first.");
      return;
    }
    const cleanUser = username.trim();
    if (!cleanUser) return;
    try {
      this.isLoadingSkin = true;
      const url = `https://minotar.net/skin/${encodeURIComponent(cleanUser)}`;
      const resp = await fetch(url);
      if (!resp.ok) throw new Error("Player skin not found");
      const blob = await resp.blob();
      const reader = new FileReader();
      reader.onload = async () => {
        const b64 = reader.result as string;
        await this.applySkinBase64(b64);
        this.showToast(`Imported ${cleanUser}'s skin!`);
      };
      reader.readAsDataURL(blob);
    } catch (e: any) {
      this.showToast(`Could not load skin for ${cleanUser}: ${e.message || e}`);
      this.isLoadingSkin = false;
    }
  }

  async resetPlayerSkin() {
    if (!this.activeAccount) return;
    try {
      const res: PlayerSkinPayload = await TauriService.resetSkin(this.activeAccount.username);
      this.activeSkinDataUrl = res?.skin_data_url || DEFAULT_STEVE_SKIN_DATA_URL;
      this.showToast("Reset skin to default Steve.");
    } catch (e) {
      this.showToast("Reset error: " + e);
    }
  }

  async resetPlayerCape() {
    if (!this.activeAccount) return;
    try {
      await TauriService.resetCape(this.activeAccount.username);
      this.activeCapeDataUrl = null;
      this.showToast("Removed player cape.");
    } catch (e) {
      this.showToast("Reset error: " + e);
    }
  }

  // --- Hardware & Java ---
  async checkHardwareAndJava() {
    try {
      this.detectedJavas = await invoke("detect_java");
    } catch (_) { }
    try {
      this.detectedGpus = await invoke("detect_gpus");
    } catch (_) { }
  }

  async loadSettings() {
    try {
      this.settings = await invoke("get_settings");
    } catch (_) { }
  }

  async saveSettings() {
    try {
      await invoke("save_settings", { settings: this.settings });
      this.showToast("Settings saved successfully!");
    } catch (e) {
      this.showToast("Failed to save settings: " + e);
    }
  }

  async pickGameDirectory() {
    try {
      const picked = await TauriService.pickGameDirectory();
      if (picked) {
        this.settings.data_dir = picked;
        await this.saveSettings();
        await this.refreshInstances();
        this.showToast(`Game directory updated to: ${picked}`);
      }
    } catch (e: any) {
      this.showToast("Failed to change game directory: " + e);
    }
  }

  async resetGameDirectory() {
    try {
      const res = await TauriService.resetGameDirectory();
      this.settings = res;
      await this.refreshInstances();
      this.showToast("Game directory reset to default system path.");
    } catch (e: any) {
      this.showToast("Failed to reset directory: " + e);
    }
  }

  async pickInstancesDirectory() {
    try {
      const picked = await TauriService.pickInstancesDirectory();
      if (picked) {
        this.settings.instances_dir = picked;
        await this.saveSettings();
        await this.refreshInstances();
        this.showToast(`Instances directory updated to: ${picked}`);
      }
    } catch (e: any) {
      this.showToast("Failed to change instances directory: " + e);
    }
  }

  async resetInstancesDirectory() {
    try {
      const res = await TauriService.resetInstancesDirectory();
      this.settings = res;
      await this.refreshInstances();
      this.showToast("Instances directory reset to default path.");
    } catch (e: any) {
      this.showToast("Failed to reset instances directory: " + e);
    }
  }

  async selectJava(path: string | null) {
    this.settings.custom_java_path = path;
    await this.saveSettings();
    if (path) {
      const found = this.detectedJavas.find((j) => j.path === path);
      this.showToast(`Active Java set to: ${found?.display_name || path}`);
    } else {
      this.showToast("Java runtime set to Auto-Detect (Smart Version Matching).");
    }
  }

  async scanForJava() {
    try {
      this.detectedJavas = await invoke("detect_java");
      this.showToast(`Scanned system: ${this.detectedJavas.length} Java runtimes found.`);
    } catch (e: any) {
      this.showToast("Failed to scan Java: " + e);
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
    this.crashDetails = null;
    this.hasManualCrash = false;

    try {
      await invoke("launch_game", { instanceId: this.activeInstance.id });
    } catch (e: any) {
      this.gameStatus = "crashed";
      this.hasManualCrash = true;
      const errStr = typeof e === "string" ? e : (e?.message || JSON.stringify(e));
      this.launchStatusText = `Launch failed: ${errStr}`;
      this.showToast(`Error: ${errStr}`);
      try {
        const lastCrash: CrashAnalysis | null = await invoke("get_last_crash_analysis");
        if (lastCrash) {
          this.crashDetails = lastCrash;
        }
      } catch (_) {}
    }
  }

  // --- Log & Crash Management ---
  get hasCrashDetected(): boolean {
    if (this.hasManualCrash || this.crashDetails !== null) return true;
    return this.consoleLogs.some(l =>
      l.includes("CrashReport") ||
      l.includes("FATAL") ||
      l.includes("A fatal error has been detected") ||
      l.includes("Minecraft has crashed") ||
      l.includes("Shutting down with exit code 1") ||
      l.includes("Shutting down with exit code -1") ||
      l.includes("Caused by:") ||
      l.includes("[DV ERROR DETECTOR]")
    );
  }

  get crashReportSnippet(): string {
    const crashLines: string[] = [];
    let recording = false;
    for (let i = 0; i < this.consoleLogs.length; i++) {
      const line = this.consoleLogs[i];
      if (
        line.includes("Exception") ||
        line.includes("CrashReport") ||
        line.includes("FATAL") ||
        line.includes("Caused by:") ||
        line.includes("A fatal error has been detected")
      ) {
        recording = true;
      }
      if (recording) {
        crashLines.push(line);
        if (crashLines.length >= 40) break;
      }
    }
    return crashLines.join("\n");
  }

  async loadLatestLogs() {
    try {
      const content: string = await invoke("get_latest_logs", {
        instanceId: this.activeInstance?.id || null,
      });
      if (content && content.trim()) {
        const lines = content.split(/\r?\n/).filter(l => l.length > 0);
        this.consoleLogs = lines.slice(-2500);
        this.showToast(`Loaded ${lines.length} log lines from disk.`);
      } else {
        this.showToast("No latest.log found on disk for this profile yet.");
      }
    } catch (e) {
      this.showToast("Failed to load logs: " + e);
    }
  }

  async openLogsFolder() {
    try {
      await invoke("open_logs_folder", {
        instanceId: this.activeInstance?.id || null,
      });
    } catch (e) {
      this.showToast("Failed to open logs folder: " + e);
    }
  }

  async exportLogs() {
    if (this.consoleLogs.length === 0) {
      this.showToast("No log events to export.");
      return;
    }
    try {
      const saved: string | null = await invoke("export_logs", {
        content: this.consoleLogs.join("\n"),
      });
      if (saved) {
        this.showToast(`Log saved to ${saved}`);
      }
    } catch (e) {
      this.showToast("Export error: " + e);
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

  async reportCrash(): Promise<boolean> {
    if (this.isReportingCrash) return false;
    this.isReportingCrash = true;
    try {
      const logsToSend = this.consoleLogs.join("\n");
      const snippet = this.crashReportSnippet || null;
      await TauriService.reportCrash({
        logs: logsToSend,
        crashSnippet: snippet,
        instanceId: this.activeInstance?.id || null,
      });
      this.crashReportSuccess = true;
      this.showToast("Report submitted successfully. Thank you!");
      setTimeout(() => {
        this.crashReportSuccess = false;
      }, 5000);
      return true;
    } catch (e: any) {
      console.error("Failed to submit report:", e);
      this.showToast(`Report submission failed: ${e?.message || e}`);
      return false;
    } finally {
      this.isReportingCrash = false;
    }
  }

  // --- Software Updates ---
  async checkForUpdates(userInitiated = false): Promise<void> {
    if (this.isCheckingUpdates) return;
    this.isCheckingUpdates = true;
    try {
      const res: UpdateCheckResult = await TauriService.checkForUpdates();
      this.updateInfo = res;
      if (res.has_update) {
        if (userInitiated) {
          this.showUpdateModal = true;
          this.showToast(`New update available: ${res.latest_version}!`);
        }
      } else if (userInitiated) {
        this.showToast(`You are running the latest version (v${res.current_version}).`);
      }
    } catch (e: any) {
      if (userInitiated) {
        this.showToast(`Failed to check for updates: ${e?.message || e}`);
      }
      console.warn("Update check failed:", e);
    } finally {
      this.isCheckingUpdates = false;
    }
  }

  async openReleaseUrl(): Promise<void> {
    const url =
      this.updateInfo?.download_url ||
      this.updateInfo?.html_url ||
      "https://github.com/SYN606/dv-client/releases";
    try {
      await TauriService.openUrl(url);
    } catch (e) {
      this.showToast("Failed to open update URL: " + e);
    }
  }
}

export const launcher = new LauncherState();
