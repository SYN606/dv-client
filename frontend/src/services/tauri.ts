/**
 * Centralized Tauri IPC Service Layer.
 * Provides strongly-typed asynchronous client methods to invoke Rust backend capabilities.
 */

import { invoke } from "@tauri-apps/api/core";
import type {
  LauncherInfo,
  Account,
  Instance,
  MinecraftVersionSummary,
  ModItem,
  LauncherSettings,
  JavaInstallation,
  GpuDevice,
  PlayerSkinPayload,
  UpdateCheckResult,
} from "../types";

export const TauriService = {
  // --- Launcher Info ---
  getLauncherInfo(): Promise<LauncherInfo> {
    return invoke("get_launcher_info");
  },

  // --- Accounts ---
  getAccounts(): Promise<Account[]> {
    return invoke("get_accounts");
  },

  selectActiveAccount(accountId: string): Promise<void> {
    return invoke("select_active_account", { accountId });
  },

  addOfflineAccount(username: string): Promise<Account> {
    return invoke("add_offline_account", { username });
  },

  logoutAccount(accountId: string): Promise<void> {
    return invoke("logout_account", { accountId });
  },

  startMicrosoftLogin(): Promise<{ auth_url: string; flow_id: string }> {
    return invoke("start_microsoft_login");
  },

  completeMicrosoftLogin(flowId: string, authCode: string): Promise<Account> {
    return invoke("complete_microsoft_login", { flowId, authCode });
  },

  // --- Instances & Versions ---
  getInstances(): Promise<Instance[]> {
    return invoke("get_instances");
  },

  selectActiveInstance(instanceId: string): Promise<void> {
    return invoke("select_active_instance", { instanceId });
  },

  createInstance(name: string, mcVersion: string, modLoader: string): Promise<Instance> {
    return invoke("create_instance", { name, mcVersion, modLoader });
  },

  deleteInstance(instanceId: string): Promise<void> {
    return invoke("delete_instance", { instanceId });
  },

  getAllMinecraftVersions(): Promise<MinecraftVersionSummary[]> {
    return invoke("get_all_minecraft_versions");
  },

  // --- Mod Management ---
  getInstalledMods(instanceId: string): Promise<ModItem[]> {
    return invoke("get_installed_mods", { instanceId });
  },

  toggleMod(instanceId: string, filename: string, enabled: boolean): Promise<void> {
    return invoke("toggle_mod", { instanceId, filename, enabled });
  },

  deleteMod(instanceId: string, filename: string): Promise<void> {
    return invoke("delete_mod", { instanceId, filename });
  },

  bulkToggleMods(instanceId: string, enableAll: boolean): Promise<number> {
    return invoke("bulk_toggle_mods", { instanceId, enableAll });
  },

  importLocalMod(instanceId: string): Promise<string | null> {
    return invoke("import_local_mod", { instanceId });
  },

  searchModrinth(params: {
    query: string | null;
    loader: string | null;
    gameVersion: string;
    category: string | null;
    sortBy: string;
    limit: number;
  }): Promise<any[]> {
    return invoke("search_modrinth", params);
  },

  installModrinthMod(instanceId: string, projectId: string): Promise<string> {
    return invoke("install_modrinth_mod", { instanceId, projectId });
  },

  uninstallModrinthMod(instanceId: string, slug: string): Promise<void> {
    return invoke("uninstall_modrinth_mod", { instanceId, slug });
  },

  searchCurseforge(params: {
    query: string | null;
    loader: string | null;
    gameVersion: string;
    categoryId: number | null;
    sortBy: string;
    limit: number;
  }): Promise<any[]> {
    return invoke("search_curseforge", params);
  },

  installCurseforgeMod(instanceId: string, modId: number): Promise<string> {
    return invoke("install_curseforge_mod", { instanceId, modId });
  },

  openModsFolder(instanceId?: string | null): Promise<void> {
    return invoke("open_mods_folder", { instanceId: instanceId || null });
  },

  // --- Skins & Capes ---
  getPlayerSkin(username?: string | null): Promise<PlayerSkinPayload> {
    return invoke("get_player_skin", { username: username || null });
  },

  pickSkinFile(): Promise<string | null> {
    return invoke("pick_skin_file");
  },

  pickCapeFile(): Promise<string | null> {
    return invoke("pick_cape_file");
  },

  applySkin(username: string, filePath: string): Promise<PlayerSkinPayload> {
    return invoke("apply_skin", { username, filePath });
  },

  applySkinBase64(username: string, base64Data: string): Promise<PlayerSkinPayload> {
    return invoke("apply_skin_base64", { username, base64Data });
  },

  applyCape(username: string, filePath: string): Promise<PlayerSkinPayload> {
    return invoke("apply_cape", { username, filePath });
  },

  applyCapeBase64(username: string, base64Data: string): Promise<PlayerSkinPayload> {
    return invoke("apply_cape_base64", { username, base64Data });
  },

  resetSkin(username: string): Promise<PlayerSkinPayload> {
    return invoke("reset_skin", { username });
  },

  resetCape(username: string): Promise<PlayerSkinPayload> {
    return invoke("reset_cape", { username });
  },

  // --- Game Lifecycle ---
  launchGame(instanceId: string): Promise<void> {
    return invoke("launch_game", { instanceId });
  },

  stopGame(): Promise<void> {
    return invoke("stop_game");
  },

  getGameStatus(): Promise<string> {
    return invoke("get_game_status");
  },

  getLatestLogs(instanceId?: string | null): Promise<string> {
    return invoke("get_latest_logs", { instanceId: instanceId || null });
  },

  openLogsFolder(instanceId?: string | null): Promise<void> {
    return invoke("open_logs_folder", { instanceId: instanceId || null });
  },

  exportLogs(content: string): Promise<string | null> {
    return invoke("export_logs", { content });
  },

  reportCrash(params: {
    logs: string;
    crashSnippet?: string | null;
    instanceId?: string | null;
  }): Promise<boolean> {
    return invoke("report_crash", {
      logs: params.logs,
      crashSnippet: params.crashSnippet ?? null,
      instanceId: params.instanceId ?? null,
    });
  },

  // --- Hardware, Java & Settings ---
  detectJava(): Promise<JavaInstallation[]> {
    return invoke("detect_java");
  },

  installJava(majorVersion: number): Promise<string> {
    return invoke("install_java", { majorVersion });
  },

  pickCustomJava(): Promise<JavaInstallation | null> {
    return invoke("pick_custom_java");
  },

  detectGpus(): Promise<GpuDevice[]> {
    return invoke("detect_gpus");
  },

  getSettings(): Promise<LauncherSettings> {
    return invoke("get_settings");
  },

  saveSettings(settings: LauncherSettings): Promise<void> {
    return invoke("save_settings", { settings });
  },

  pickGameDirectory(): Promise<string | null> {
    return invoke("pick_game_directory");
  },

  resetGameDirectory(): Promise<LauncherSettings> {
    return invoke("reset_game_directory");
  },

  pickInstancesDirectory(): Promise<string | null> {
    return invoke("pick_instances_directory");
  },

  resetInstancesDirectory(): Promise<LauncherSettings> {
    return invoke("reset_instances_directory");
  },

  openFolder(path: string): Promise<void> {
    return invoke("open_folder", { path });
  },

  checkForUpdates(): Promise<UpdateCheckResult> {
    return invoke("check_for_updates");
  },

  openUrl(url: string): Promise<void> {
    return invoke("open_url", { url });
  },
};
