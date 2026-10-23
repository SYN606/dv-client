/**
 * Minecraft Versions, Mod Loaders, and Catalog Presets.
 */

export interface PopularVersionItem {
  ver: string;
  label: string;
  desc: string;
  tag: string;
}

export const POPULAR_MINECRAFT_VERSIONS: PopularVersionItem[] = [
  { ver: "1.21.4", label: "1.21.4", desc: "Latest Tricky Trials", tag: "Latest" },
  { ver: "1.20.1", label: "1.20.1", desc: "Huge Mod Ecosystem", tag: "Popular" },
  { ver: "1.19.4", label: "1.19.4", desc: "Wild Update Era", tag: "Stable" },
  { ver: "1.16.5", label: "1.16.5", desc: "Nether Update Era", tag: "Classic" },
  { ver: "1.12.2", label: "1.12.2", desc: "Golden Modpacks Era", tag: "Legacy" },
  { ver: "1.8.9", label: "1.8.9", desc: "Competitive PvP / Hypixel", tag: "PvP" },
];

export interface ModLoaderItem {
  id: string;
  label: string;
  desc: string;
  color: string;
}

export const SUPPORTED_MOD_LOADERS: ModLoaderItem[] = [
  {
    id: "Fabric",
    label: "Fabric",
    desc: "Fastest engine, Sodium compatible",
    color: "border-purple-500/40 text-purple-300",
  },
  {
    id: "Vanilla",
    label: "Vanilla",
    desc: "Official unmodded Minecraft",
    color: "border-white/20 text-neutral-200",
  },
  {
    id: "NeoForge",
    label: "NeoForge",
    desc: "Modern Forge fork for 1.20.2+",
    color: "border-orange-500/40 text-orange-300",
  },
  {
    id: "Forge",
    label: "Forge",
    desc: "Traditional Minecraft Forge loader",
    color: "border-amber-500/40 text-amber-300",
  },
];

export const MOD_CATEGORIES = [
  { id: "all", label: "All Mods" },
  { id: "optimization", label: "Optimization" },
  { id: "utility", label: "Utility" },
  { id: "decoration", label: "Decoration" },
  { id: "technology", label: "Technology" },
  { id: "equipment", label: "Equipment" },
  { id: "adventure", label: "Adventure" },
  { id: "library", label: "Library" },
];

export const MOD_SORT_OPTIONS = [
  { id: "downloads", label: "Most Popular" },
  { id: "follows", label: "Most Followed" },
  { id: "updated", label: "Recently Updated" },
  { id: "newest", label: "Newest" },
];
