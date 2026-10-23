export type Tab = "play" | "instances" | "mods" | "skins" | "settings" | "console";

export interface LauncherInfo {
  name: string;
  version: string;
  author: string;
  repo_url: string;
}

export interface Instance {
  id: string;
  name: string;
  mc_version: string;
  mod_loader: string; // "Fabric" | "Vanilla" | "NeoForge" | "Forge"
  last_played: number | null;
  min_memory_mb: number;
  max_memory_mb: number;
  is_active: boolean;
}

export interface Account {
  id: string;
  username: string;
  account_type: "microsoft" | "offline" | string;
  avatar_url: string | null;
  last_login: number;
  is_active: boolean;
}

export interface JavaInstallation {
  path: string;
  vendor: string;
  version: string;
  major_version: number;
  display_name: string;
  is_recommended: boolean;
}

export interface GpuDevice {
  id: string;
  name: string;
  is_discrete: boolean;
}

export interface LauncherSettings {
  default_min_memory_mb: number;
  default_max_memory_mb: number;
  default_window_width: number;
  default_window_height: number;
  selected_gpu: string | null;
  custom_jvm_args: string;
  performance_preset: string;
  on_launch_action: string;
  enable_discord_rpc: boolean;
  data_dir: string;
  instances_dir: string;
  custom_java_path: string | null;
}

export interface ModItem {
  file_name: string;
  name: string;
  version: string;
  description: string;
  author: string;
  file_size_bytes: number;
  file_size_display: string;
  is_enabled: boolean;
  icon_path: string | null;
  filename?: string;
  clean_name?: string;
}

export interface DownloadProgress {
  current: number;
  total: number;
  percent: number;
  status: string;
  bytes_downloaded?: number;
  total_bytes?: number;
}

export type GameExecutionState = "idle" | "launching" | "downloading" | "running";

export interface MinecraftVersionSummary {
  id: string;
  version_type: string; // "release" | "snapshot" | "old_beta" | "old_alpha"
  release_time: string;
  is_latest_release: boolean;
  is_latest_snapshot: boolean;
}

export type ModProvider = "modrinth" | "curseforge";

export interface CatalogModItem {
  provider: ModProvider;
  id: string;
  slug: string;
  title: string;
  description: string;
  author: string;
  icon_url: string | null;
  downloads: number;
  categories: string[];
  website_url?: string;
}

export interface PlayerSkinPayload {
  skin_data_url: string;
  cape_data_url: string | null;
}
