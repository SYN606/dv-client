/**
 * 3D Skin & Cape Configuration and Offline Fallback Textures.
 * Provides instant, zero-latency 64x64 RGBA textures with no CORS restrictions.
 */

export const DEFAULT_STEVE_SKIN_DATA_URL =
  "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAEAAAABACAYAAACqaXHeAAABjklEQVR4Ae3AA6AkWZbG8f937o3IzKdyS2Oubdu2bdu2bdu2bWmMnpZKr54yMyLu+Xa3anqmhztr1S/4/43g/zeC/98I/n8j+P+N4P83gv/fCP5/o/IveLOXOG1eiF/4u3Pify+C/98I/n+j8i/4uHd4Q16YX/i7H+R/MYIX0Yf+6htyvw/91Tfk/wgqL6JvfMNf5X7f+Ia/yv8RBP+/Efz/hvj/jeD/N4L/3wj+f6PyXB79Eh9pHuAJf/e14l/ju3/evDDv/ebi3+O7f9480Hu/ufi3I/j/jeD/N4L/36j8L/Obt/0gD/S6/LsQ/P9G8P8bwf9vBP+/Efz/RvD/G8H/b4j/3wj+fyP4/43g/zeC/98I/n8j+P+N4P83gv/fCP5/I/j/jeD/N4L/3wj+fyP4/43g/zeC/98I/n8j+P+N4P83Kv/BHv0SH2ke4Al/97XiX+O7f9480Hu/ufjPQ/D/G8H/bwT/v1H5H+Y3b/tBHuh1+U9F8P8bwf9vBP+/Efz/RvD/G8H/bwT/v/GP7egj2U0hoT8AAAAASUVORK5CYII=";

export const CYBER_AGENT_SKIN_DATA_URL =
  "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAEAAAABACAYAAACqaXHeAAABN0lEQVR4nO3aMWoCQRjF8d1gwiJoJEVgG7G2sPAQ06SxzBk8gGcQD5AzpFwCacRTWIiliOANbE2VzswHmRn/yrxf6Q7O4/HtIOuW/cHwXID2u01J7v9Abn4LVAAdgKYC6AA0FUAHoLWsBR+zd+/16eIzWhiCJsBa8DZv/AvuvEKzgMI5//XVV6QoDLOA73Hbv2AVKwrjzgc4nAqgA9DK595r0POAx6cqVpaLqso4gwyH/db7vCH7CVABdABa9gXYvwT/Yb04Xfx8NAs70FLIfgKSFlC7SVG7ScotgiW5BX4dl03Kr48i+1sgyQTc4mH3l+wnQAXQAWgqgA4gIiIiIiIiInJtwe/qh75f0Om+BO1v/f9vyf6JkAqgA9BUAB2ApgLoADQVQAegqQA6AE0F0AFoKoAOQPsB8owb9frj86sAAAAASUVORK5CYII=";

export const ENDER_HOOD_SKIN_DATA_URL =
  "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAEAAAABACAYAAACqaXHeAAAA60lEQVR4nO3aMQuCQACGYS8szMEhWtr82f4af45tHbQUJdTWFCqc9Ra+zyZ63MeHHNxxYVcdHhnodD4Gcv4VOfkvsAA6AM0C6AA0C6AD0PKxD663y+D7YrOdLQxh8X+ABUz5qK1j1jXx9dw1MWvrODDif4SxvcCn1wD3AjALoAPQQlXuk84DiqKcK8tbfX9PGj+2xiz+D7AAOgDNAugANAugA9AsgA5AswA6AM0C6AA0C6ADSJIkSZIkfVvy/ZzU+wV5vk6aP/WO0eJPhCyADkCzADoAzQLoADQLoAPQLIAOQLMAOgDNAugAtCfu4SHToaFp2wAAAABJRU5ErkJggg==";

export const DV_CRIMSON_CAPE_DATA_URL =
  "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAEAAAAAgCAYAAACinX6EAAAAy0lEQVR4nO3WsQ6CMBgE4MMwNY666urEE7jgioNTH5Ghs51ZfBFccTRdcdLBGCVt4afxvokEaC4HlD87rzY9RnC8XbMx1o0tB4CLuw+6eK+WKI0atvLBO9Okcp+bGu2+nh9c0gwspANIYwHSAaQFF1AaldQ3/y6ogNIoNNqh0S7ZEoIKULvTx+OUeP0Gn2xRv568LeoogaYWVADweyaYu+BNsOpaVF0bI4uI4DfArrcxcojhHCAdQNrfF+C1B6Q69BARERERERERAOAB/sEgcc3flU4AAAAASUVORK5CYII=";

export const FOUNDER_GOLD_CAPE_DATA_URL =
  "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAEAAAAAgCAYAAACinX6EAAAApUlEQVR4nO3XsQ3CMBSE4QuCIUxBhmARM4SnyhDJIgxhCtJkAwqoQhMpUmKbJ4v/ay0/na+w5ebZX94q4Hx7NCXm5nbcuuEaTiVymNlcgCTdu9fqek0lHawDWKMA6wDWKCB1gPNRzsccWUwkFeB8lKYgTaHaEnY9g7NxaL8HH4c2S6BfSypAqvfgMy5B6wDWKMA6gLW/L2DXK1DTbw8AAAAAACx8ALEUFpxLtLzJAAAAAElFTkSuQmCC";

export const ENDER_VOID_CAPE_DATA_URL =
  "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAEAAAAAgCAYAAACinX6EAAAAoElEQVR4nO3WIQ7CMBjF8a9sEoVbsgQxzzF6AJKZHoQLzO0MO8OCnp7gIGgUljAxhf46ntj/p1rR5vXlEw2nY/21Dbzez7DFvbkd1AHUKEAdQI0C1AHUSs/hz+36sy/60RVGwT0BVYpWpZgji4RrAszMHs36+LNN7jAK7gKGy7wuWu9NGu4CuvaeIYZO4Cu8cxSgDqBGAeoAAAAAAADg7xZDYRIQM+z/wQAAAABJRU5ErkJggg==";

export const CYBER_NEON_CAPE_DATA_URL =
  "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAEAAAAAgCAYAAACinX6EAAAAfElEQVR4nO3WsQ2AIBSEYbBGK1nEwRzGCZyIRbAy9NibaCGSk/h/DQkJ5HLFy7NuGLOpIO2brfHv2zp1ADUKePIohXh7tsSWzoAUosmLN/16umcGtIEC1AHUKEAdQI0C1AHUKEAdQK14FXaTN3b+8Sp8hVUYAAAAAAB81QFSvSWIcV13cAAAAABJRU5ErkJggg==";

export type SkinModelType = "classic" | "slim";
export type SkinAnimationType = "walk" | "run" | "idle" | "fly";

export interface SkinStudioState {
  modelType: SkinModelType;
  animation: SkinAnimationType;
  speed: number;
}

export interface SkinPreset {
  id: string;
  name: string;
  model: SkinModelType;
  tag: string;
  color: string;
  dataUrl: string;
}

export interface CapePreset {
  id: string;
  name: string;
  rarity: "Exclusive" | "Legendary" | "Mythic" | "Classic";
  tag: string;
  color: string;
  dataUrl: string;
}

export const SKIN_PRESETS: SkinPreset[] = [
  {
    id: "steve",
    name: "Classic Steve",
    model: "classic",
    tag: "Official Vanilla",
    color: "from-cyan-500/20 to-blue-500/10 border-cyan-500/30 text-cyan-300",
    dataUrl: DEFAULT_STEVE_SKIN_DATA_URL,
  },
  {
    id: "cyber-agent",
    name: "Cyber DV Agent",
    model: "classic",
    tag: "Digital Vigital Elite",
    color: "from-red-500/20 to-amber-500/10 border-red-500/30 text-red-300",
    dataUrl: CYBER_AGENT_SKIN_DATA_URL,
  },
  {
    id: "ender-hood",
    name: "Ender Void Robe",
    model: "classic",
    tag: "Voidwalker Series",
    color: "from-purple-500/20 to-indigo-500/10 border-purple-500/30 text-purple-300",
    dataUrl: ENDER_HOOD_SKIN_DATA_URL,
  },
];

export const CAPE_PRESETS: CapePreset[] = [
  {
    id: "dv-crimson",
    name: "DV Crimson & Gold",
    rarity: "Exclusive",
    tag: "Digital Vigital Official",
    color: "from-red-600/30 to-amber-500/20 border-red-500/40 text-red-300",
    dataUrl: DV_CRIMSON_CAPE_DATA_URL,
  },
  {
    id: "founder-gold",
    name: "Founder Obsidian Gold",
    rarity: "Legendary",
    tag: "VIP Founder Star",
    color: "from-amber-500/30 to-yellow-500/20 border-amber-500/40 text-amber-300",
    dataUrl: FOUNDER_GOLD_CAPE_DATA_URL,
  },
  {
    id: "ender-void",
    name: "Ender Void Eye",
    rarity: "Mythic",
    tag: "The End Dimension",
    color: "from-purple-600/30 to-emerald-500/20 border-purple-500/40 text-purple-300",
    dataUrl: ENDER_VOID_CAPE_DATA_URL,
  },
  {
    id: "cyber-neon",
    name: "Cyber Neon Circuit",
    rarity: "Exclusive",
    tag: "High-Tech Hex",
    color: "from-cyan-500/30 to-blue-500/20 border-cyan-500/40 text-cyan-300",
    dataUrl: CYBER_NEON_CAPE_DATA_URL,
  },
];
