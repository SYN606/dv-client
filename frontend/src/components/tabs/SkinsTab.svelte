<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { launcher } from "../../state/launcher.svelte";
  import * as skinview3d from "skinview3d";
  import {
    User,
    Upload,
    Shield,
    RotateCcw,
    Info,
    Layers,
    Sparkles,
    Trash2,
    Activity,
    Check,
  } from "lucide-svelte";

  let canvasEl: HTMLCanvasElement | null = $state(null);
  let viewer: skinview3d.SkinViewer | null = $state(null);
  let animationMode = $state<"walk" | "run" | "idle">("walk");
  let modelType = $state<"classic" | "slim">("classic");

  function initViewer() {
    if (!canvasEl) return;
    if (viewer) {
      viewer.dispose();
      viewer = null;
    }

    try {
      const skinSource = launcher.activeSkinDataUrl;
      const model = modelType === "slim" ? "slim" : "default";

      viewer = new skinview3d.SkinViewer({
        canvas: canvasEl,
        width: 320,
        height: 440,
        skin: skinSource,
        model: model,
      });

      // Camera lighting and position
      viewer.fov = 70;
      viewer.zoom = 0.9;
      viewer.globalLight.intensity = 0.85;
      viewer.cameraLight.intensity = 0.65;

      // Apply initial animation
      applyAnimation(animationMode);

      // Apply cape if active
      if (launcher.activeCapeDataUrl) {
        viewer.loadCape(launcher.activeCapeDataUrl);
      }
    } catch (e) {
      console.warn("Could not initialize 3D skin viewer:", e);
    }
  }

  function applyAnimation(mode: "walk" | "run" | "idle") {
    if (!viewer) return;
    animationMode = mode;
    if (mode === "walk") {
      viewer.animation = new skinview3d.WalkingAnimation();
      viewer.animation.speed = 0.7;
    } else if (mode === "run") {
      viewer.animation = new skinview3d.RunningAnimation();
      viewer.animation.speed = 0.9;
    } else {
      viewer.animation = null;
    }
  }

  function setModelType(type: "classic" | "slim") {
    modelType = type;
    launcher.skinModelType = type;
    if (viewer) {
      viewer.loadSkin(launcher.activeSkinDataUrl, {
        model: type === "slim" ? "slim" : "default",
      });
    }
  }

  // Reactively update 3D skin & cape when launcher state changes
  $effect(() => {
    const skin = launcher.activeSkinDataUrl;
    if (viewer && skin) {
      viewer.loadSkin(skin, {
        model: modelType === "slim" ? "slim" : "default",
      });
    }
  });

  $effect(() => {
    const cape = launcher.activeCapeDataUrl;
    if (viewer) {
      if (cape) {
        viewer.loadCape(cape);
      } else {
        viewer.loadCape(null);
      }
    }
  });

  onMount(() => {
    initViewer();
    if (!launcher.activeSkinDataUrl) {
      launcher.loadPlayerSkin();
    }
  });

  onDestroy(() => {
    if (viewer) {
      viewer.dispose();
      viewer = null;
    }
  });

  async function handleApplySkin() {
    await launcher.pickAndApplySkin();
  }

  async function handleApplyCape() {
    await launcher.pickAndApplyCape();
  }

  async function handleResetSkin() {
    await launcher.resetPlayerSkin();
  }

  async function handleResetCape() {
    await launcher.resetPlayerCape();
  }
</script>

<div class="h-full overflow-y-auto p-8 space-y-6">
  <!-- Top Bar -->
  <div class="flex items-center justify-between pb-4 border-b border-white/8">
    <div>
      <h2
        class="text-2xl font-black text-white uppercase tracking-tight flex items-center gap-3"
      >
        <User class="w-6 h-6 text-red-500" />
        <span>3D Skins & Capes Studio</span>
      </h2>
      <p class="text-xs text-neutral-300 mt-1 font-medium">
        Customize your Minecraft player skin and cosmetic cape with offline-capable real-time 3D rendering.
      </p>
    </div>
  </div>

  <div class="grid grid-cols-1 lg:grid-cols-12 gap-8 items-start">
    <!-- 3D Canvas Box -->
    <div class="lg:col-span-6 flex flex-col items-center">
      <div
        class="relative rounded-3xl overflow-hidden border border-white/10 bg-gradient-to-b from-[#141724]/70 to-[#0b0c13] p-5 shadow-2xl flex flex-col items-center w-full max-w-sm"
      >
        <!-- Ambient Red Specular Flare -->
        <div
          class="absolute -top-12 -left-12 w-48 h-48 bg-red-600/10 rounded-full blur-2xl pointer-events-none"
        ></div>

        <div class="relative w-full flex items-center justify-center">
          <canvas
            bind:this={canvasEl}
            class="rounded-2xl cursor-grab active:cursor-grabbing shadow-inner"
          ></canvas>

          {#if launcher.isLoadingSkin}
            <div
              class="absolute inset-0 bg-black/60 backdrop-blur-sm rounded-2xl flex flex-col items-center justify-center gap-2 text-xs text-white"
            >
              <div
                class="w-6 h-6 border-2 border-red-500/30 border-t-red-500 rounded-full animate-spin"
              ></div>
              <span>Updating 3D Textures...</span>
            </div>
          {/if}
        </div>

        <!-- Controls: Animation Selector -->
        <div class="mt-4 w-full space-y-3">
          <div class="flex items-center justify-between gap-1 bg-white/4 p-1 rounded-xl border border-white/6">
            <button
              type="button"
              onclick={() => applyAnimation("walk")}
              class="flex-1 py-1.5 rounded-lg text-xs font-bold transition-all cursor-pointer {animationMode === 'walk'
                ? 'bg-red-600 text-white shadow-md'
                : 'text-neutral-400 hover:text-white'}"
            >
              Walk
            </button>
            <button
              type="button"
              onclick={() => applyAnimation("run")}
              class="flex-1 py-1.5 rounded-lg text-xs font-bold transition-all cursor-pointer {animationMode === 'run'
                ? 'bg-red-600 text-white shadow-md'
                : 'text-neutral-400 hover:text-white'}"
            >
              Run
            </button>
            <button
              type="button"
              onclick={() => applyAnimation("idle")}
              class="flex-1 py-1.5 rounded-lg text-xs font-bold transition-all cursor-pointer {animationMode === 'idle'
                ? 'bg-red-600 text-white shadow-md'
                : 'text-neutral-400 hover:text-white'}"
            >
              Idle
            </button>
          </div>

          <!-- Model Arm Type: Classic (4px) vs Slim (3px) -->
          <div class="flex items-center justify-between gap-2">
            <span class="text-xs font-bold uppercase text-neutral-400">Model:</span>
            <div class="flex items-center gap-1 bg-white/4 p-1 rounded-xl border border-white/6">
              <button
                type="button"
                onclick={() => setModelType("classic")}
                class="px-3 py-1 rounded-lg text-xs font-bold transition-all cursor-pointer {modelType === 'classic'
                  ? 'bg-white/15 text-white border border-white/20'
                  : 'text-neutral-400 hover:text-white'}"
              >
                Classic (4px)
              </button>
              <button
                type="button"
                onclick={() => setModelType("slim")}
                class="px-3 py-1 rounded-lg text-xs font-bold transition-all cursor-pointer {modelType === 'slim'
                  ? 'bg-white/15 text-white border border-white/20'
                  : 'text-neutral-400 hover:text-white'}"
              >
                Slim (3px)
              </button>
            </div>

            <button
              type="button"
              onclick={initViewer}
              title="Reset 3D camera angle"
              class="p-2 rounded-xl bg-white/5 hover:bg-white/10 border border-white/10 text-neutral-300 hover:text-white transition-all cursor-pointer"
            >
              <RotateCcw class="w-3.5 h-3.5" />
            </button>
          </div>
        </div>
      </div>
    </div>

    <!-- Actions & Info Cards -->
    <div class="lg:col-span-6 space-y-5">
      <!-- Active Account Tag -->
      <div
        class="p-5 rounded-2xl bg-[#0e1017]/90 border border-white/8 space-y-4"
      >
        <h3
          class="text-sm font-bold text-white uppercase tracking-wider flex items-center gap-2"
        >
          <Shield class="w-4 h-4 text-red-400" />
          <span>Active Player Profile</span>
        </h3>

        {#if launcher.activeAccount}
          <div
            class="flex items-center justify-between p-3.5 rounded-xl bg-white/4 border border-white/8"
          >
            <div class="flex items-center gap-3.5">
              <div
                class="w-10 h-10 rounded-lg bg-red-500/10 border border-red-500/20 flex items-center justify-center text-red-400 font-bold text-base"
              >
                {launcher.activeAccount.username.substring(0, 2).toUpperCase()}
              </div>
              <div>
                <div class="text-sm font-bold text-white">
                  {launcher.activeAccount.username}
                </div>
                <div class="text-xs text-neutral-300 font-medium">
                  {launcher.activeAccount.account_type === "microsoft"
                    ? "Microsoft Account"
                    : "Offline Profile"}
                </div>
              </div>
            </div>

            <div class="flex items-center gap-1.5">
              <span
                class="px-2 py-0.5 rounded text-[10px] font-bold uppercase bg-emerald-500/20 text-emerald-300 border border-emerald-500/30"
              >
                Active
              </span>
            </div>
          </div>
        {:else}
          <div
            class="p-3.5 text-xs text-amber-300 font-medium bg-amber-500/10 border border-amber-500/20 rounded-xl"
          >
            Please log in or add a player profile first to assign skins.
          </div>
        {/if}

        <!-- Active Texture Details -->
        <div class="grid grid-cols-2 gap-2 text-xs">
          <div class="p-2.5 rounded-xl bg-black/30 border border-white/6 space-y-1">
            <span class="text-neutral-400 block text-[11px]">Skin Texture:</span>
            <span class="text-white font-mono font-bold flex items-center gap-1.5">
              <span class="w-2 h-2 rounded-full bg-emerald-400"></span>
              <span>64x64 RGBA</span>
            </span>
          </div>

          <div class="p-2.5 rounded-xl bg-black/30 border border-white/6 space-y-1">
            <span class="text-neutral-400 block text-[11px]">Active Cape:</span>
            <span class="text-white font-mono font-bold flex items-center gap-1.5">
              <span
                class="w-2 h-2 rounded-full {launcher.activeCapeDataUrl
                  ? 'bg-purple-400'
                  : 'bg-neutral-600'}"
              ></span>
              <span>{launcher.activeCapeDataUrl ? "Equipped" : "None"}</span>
            </span>
          </div>
        </div>
      </div>

      <!-- Upload Actions -->
      <div
        class="p-5 rounded-2xl bg-[#0e1017]/90 border border-white/8 space-y-4"
      >
        <h3
          class="text-sm font-bold text-white uppercase tracking-wider flex items-center gap-2"
        >
          <Upload class="w-4 h-4 text-emerald-400" />
          <span>Upload Custom Skin & Cape</span>
        </h3>

        <div class="grid grid-cols-2 gap-3">
          <button
            type="button"
            onclick={handleApplySkin}
            class="p-4 rounded-xl bg-white/4 hover:bg-red-500/15 border border-white/8 hover:border-red-500/30 text-left transition-all cursor-pointer group"
          >
            <div
              class="w-9 h-9 rounded-lg bg-white/6 flex items-center justify-center text-neutral-200 group-hover:text-red-400 mb-2"
            >
              <Upload class="w-4 h-4" />
            </div>
            <div class="text-sm font-bold text-white group-hover:text-red-400">
              Apply Skin
            </div>
            <div class="text-xs text-neutral-400">Standard 64x64 PNG</div>
          </button>

          <button
            type="button"
            onclick={handleApplyCape}
            class="p-4 rounded-xl bg-white/4 hover:bg-red-500/15 border border-white/8 hover:border-red-500/30 text-left transition-all cursor-pointer group"
          >
            <div
              class="w-9 h-9 rounded-lg bg-white/6 flex items-center justify-center text-neutral-200 group-hover:text-red-400 mb-2"
            >
              <Layers class="w-4 h-4" />
            </div>
            <div class="text-sm font-bold text-white group-hover:text-red-400">
              Apply Cape
            </div>
            <div class="text-xs text-neutral-400">Standard 64x32 PNG</div>
          </button>
        </div>

        <!-- Reset Actions -->
        <div class="flex items-center gap-2 pt-1">
          <button
            type="button"
            onclick={handleResetSkin}
            class="flex-1 py-2 px-3 rounded-xl bg-white/4 hover:bg-white/8 border border-white/8 text-neutral-400 hover:text-white text-xs font-semibold flex items-center justify-center gap-1.5 transition-colors cursor-pointer"
          >
            <RotateCcw class="w-3.5 h-3.5" />
            <span>Reset to Steve</span>
          </button>

          {#if launcher.activeCapeDataUrl}
            <button
              type="button"
              onclick={handleResetCape}
              class="flex-1 py-2 px-3 rounded-xl bg-white/4 hover:bg-rose-500/15 border border-white/8 hover:border-rose-500/30 text-neutral-400 hover:text-rose-300 text-xs font-semibold flex items-center justify-center gap-1.5 transition-colors cursor-pointer"
            >
              <Trash2 class="w-3.5 h-3.5" />
              <span>Remove Cape</span>
            </button>
          {/if}
        </div>

        <div
          class="p-3.5 rounded-xl bg-white/3 border border-white/6 flex items-start gap-2.5 text-xs text-neutral-300 leading-relaxed"
        >
          <Info class="w-4 h-4 text-sky-400 shrink-0 mt-0.5" />
          <span>
            Skins and capes are injected automatically into the game's resource pack upon launch, visible to you in third-person view and inventory.
          </span>
        </div>
      </div>
    </div>
  </div>
</div>
