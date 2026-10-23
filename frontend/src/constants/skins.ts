/**
 * 3D Skin & Cape Configuration and Offline Fallback Textures.
 * Provides instant, zero-latency 64x64 RGBA textures with no CORS restrictions.
 */

export const DEFAULT_STEVE_SKIN_DATA_URL =
  "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAEAAAABACAYAAACqaXHeAAABjklEQVR4Ae3AA6AkWZbG8f937o3IzKdyS2Oubdu2bdu2bdu2bWmMnpZKr54yMyLu+Xa3anqmhztr1S/4/43g/zeC/98I/n8j+P+N4P83gv/fCP5/o/IveLOXOG1eiF/4u3Pify+C/98I/n+j8i/4uHd4Q16YX/i7H+R/MYIX0Yf+6htyvw/91Tfk/wgqL6JvfMNf5X7f+Ia/yv8RBP+/Efz/hvj/jeD/N4L/3wj+f6PyXB79Eh9pHuAJf/e14l/ju3/evDDv/ebi3+O7f9480Hu/ufi3I/j/jeD/N4L/36j8L/Obt/0gD/S6/LsQ/P9G8P8bwf9vBP+/Efz/RvD/G8H/b4j/3wj+fyP4/43g/zeC/98I/n8j+P+N4P83gv/fCP5/I/j/jeD/N4L/3wj+fyP4/43g/zeC/98I/n8j+P+N4P83Kv/BHv0SH2ke4Al/97XiX+O7f9480Hu/ufjPQ/D/G8H/bwT/v1H5H+Y3b/tBHuh1+U9F8P8bwf9vBP+/Efz/RvD/G8H/bwT/v/GP7egj2U0hoT8AAAAASUVORK5CYII=";

export type SkinModelType = "classic" | "slim";
export type SkinAnimationType = "walk" | "run" | "none";

export interface SkinStudioState {
  modelType: SkinModelType;
  animation: SkinAnimationType;
  speed: number;
}
