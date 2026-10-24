<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { launcher } from "../../state/launcher.svelte";
  import * as skinview3d from "skinview3d";
  import {
    SKIN_PRESETS,
    CAPE_PRESETS,
    type SkinModelType,
    type SkinAnimationType,
  } from "../../constants/skins";
  import {
    User,
    Upload,
    Shield,
    RotateCcw,
    Layers,
    Sparkles,
    Trash2,
    Compass,
    Eye,
    EyeOff,
    Download,
    Search,
    Play,
    Pause,
    Palette,
    Check,
    Loader2,
    Sparkle,
    Shirt,
    Crown,
    ArrowUpRight,
  } from "lucide-svelte";

  let canvasEl: HTMLCanvasElement | null = $state(null);
  let viewer: skinview3d.SkinViewer | null = $state(null);
  let activeSubTab = $state<"studio" | "skins" | "capes" | "stealer">("studio");

  let animationMode = $state<SkinAnimationType>("walk");
  let animationSpeed = $state(0.8);
  let isPaused = $state(false);
  let isAutoRotating = $state(false);
  let modelType = $state<SkinModelType>("classic");
  let showCapeOnModel = $state(true);

  let searchUsername = $state("");
  let isDraggingFile = $state(false);

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

      viewer.fov = 70;
      viewer.zoom = 0.92;
      viewer.globalLight.intensity = 0.9;
      viewer.cameraLight.intensity = 0.7;

      applyAnimation(animationMode);

      if (launcher.activeCapeDataUrl && showCapeOnModel) {
        viewer.loadCape(launcher.activeCapeDataUrl);
      }
    } catch (e) {
      console.warn("Could not initialize 3D skin viewer:", e);
    }
  }

  function applyAnimation(mode: SkinAnimationType) {
    if (!viewer) return;
    animationMode = mode;
    isPaused = false;

    if (mode === "walk") {
      const anim = new skinview3d.WalkingAnimation();
      anim.speed = animationSpeed;
      viewer.animation = anim;
    } else if (mode === "run") {
      const anim = new skinview3d.RunningAnimation();
      anim.speed = animationSpeed * 1.2;
      viewer.animation = anim;
    } else if (mode === "fly") {
      const anim = new skinview3d.FlyingAnimation();
      anim.speed = animationSpeed;
      viewer.animation = anim;
    } else {
      viewer.animation = null;
    }
  }

  function togglePause() {
    if (!viewer || !viewer.animation) return;
    isPaused = !isPaused;
    viewer.animation.paused = isPaused;
  }

  function toggleAutoRotate() {
    if (!viewer) return;
    isAutoRotating = !isAutoRotating;
    viewer.autoRotate = isAutoRotating;
    viewer.autoRotateSpeed = 1.2;
  }

  function setCameraAngle(angleRad: number) {
    if (!viewer) return;
    viewer.playerObject.rotation.y = angleRad;
    viewer.camera.position.set(0, 0, 42);
    viewer.camera.lookAt(0, 0, 0);
  }

  function setModelType(type: SkinModelType) {
    modelType = type;
    launcher.skinModelType = type;
    if (viewer) {
      viewer.loadSkin(launcher.activeSkinDataUrl, {
        model: type === "slim" ? "slim" : "default",
      });
    }
  }

  function toggleCapeVisibility() {
    showCapeOnModel = !showCapeOnModel;
    if (!viewer) return;
    if (showCapeOnModel && launcher.activeCapeDataUrl) {
      viewer.loadCape(launcher.activeCapeDataUrl);
    } else {
      viewer.loadCape(null);
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
      if (cape && showCapeOnModel) {
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

  // Handle Drag & Drop of skin/cape image files directly onto canvas
  function handleDragOver(e: DragEvent) {
    e.preventDefault();
    isDraggingFile = true;
  }

  function handleDragLeave() {
    isDraggingFile = false;
  }

  async function handleDrop(e: DragEvent) {
    e.preventDefault();
    isDraggingFile = false;
    if (!e.dataTransfer || !e.dataTransfer.files.length) return;

    const file = e.dataTransfer.files[0];
    if (!file.name.toLowerCase().endsWith(".png")) {
      launcher.showToast("Please drop a valid .PNG skin or cape file.");
      return;
    }

    const reader = new FileReader();
    reader.onload = async () => {
      const dataUrl = reader.result as string;
      // Heuristic: capes are usually 64x32 or 22x17, skins are 64x64 or 64x32
      const img = new Image();
      img.onload = async () => {
        if (img.width === 64 && img.height === 32 && file.name.toLowerCase().includes("cape")) {
          await launcher.applyCapeBase64(dataUrl);
        } else {
          await launcher.applySkinBase64(dataUrl);
        }
      };
      img.src = dataUrl;
    };
    reader.readAsDataURL(file);
  }

  function downloadCurrentSkin() {
    if (!launcher.activeSkinDataUrl) return;
    const a = document.createElement("a");
    a.href = launcher.activeSkinDataUrl;
    a.download = `${launcher.activeAccount?.username || "minecraft"}_skin.png`;
    a.click();
    launcher.showToast("Skin texture downloaded!");
  }
</script>

<div class="h-full overflow-y-auto p-8 space-y-6">
  <!-- Top Bar & Profile Header -->
  <div class="flex flex-col md:flex-row md:items-center justify-between gap-4 pb-4 border-b border-white/8">
    <div>
      <h2 class="text-2xl font-black text-white uppercase tracking-tight flex items-center gap-3">
        <div class="w-8 h-8 rounded-xl bg-red-500/10 border border-red-500/20 flex items-center justify-center text-red-400">
          <Shirt class="w-4 h-4" />
        </div>
        <span>Skins & Capes Studio</span>
      </h2>
      <p class="text-xs text-neutral-300 mt-1 font-medium">
        High-performance 3D player customization with real-time pose physics, cape cosmetics, and preset collections.
      </p>
    </div>

    <!-- Active Player Badge -->
    {#if launcher.activeAccount}
      <div class="flex items-center gap-3 px-4 py-2 rounded-2xl bg-white/4 border border-white/8 shadow-md">
        <div class="w-8 h-8 rounded-lg bg-red-500/15 border border-red-500/30 flex items-center justify-center text-red-300 font-black text-xs">
          {launcher.activeAccount.username.substring(0, 2).toUpperCase()}
        </div>
        <div>
          <div class="text-xs font-bold text-white flex items-center gap-1.5">
            <span>{launcher.activeAccount.username}</span>
            <span class="w-1.5 h-1.5 rounded-full bg-emerald-400"></span>
          </div>
          <div class="text-[10px] text-neutral-400 uppercase tracking-wider font-semibold">
            {launcher.activeAccount.account_type === "microsoft" ? "Microsoft" : "Offline"} Profile
          </div>
        </div>
      </div>
    {/if}
  </div>

  <!-- Studio Sub-Navigation Tabs -->
  <div class="flex items-center gap-2 border-b border-white/8 pb-3">
    <button
      type="button"
      onclick={() => (activeSubTab = "studio")}
      class="px-4 py-2 rounded-xl text-xs font-bold uppercase tracking-wider flex items-center gap-2 transition-all cursor-pointer {activeSubTab === 'studio'
        ? 'bg-red-600 text-white shadow-lg shadow-red-600/30'
        : 'bg-white/4 hover:bg-white/8 text-neutral-400 hover:text-white border border-white/6'}"
    >
      <Compass class="w-3.5 h-3.5" />
      <span>Studio & 3D Stage</span>
    </button>

    <button
      type="button"
      onclick={() => (activeSubTab = "skins")}
      class="px-4 py-2 rounded-xl text-xs font-bold uppercase tracking-wider flex items-center gap-2 transition-all cursor-pointer {activeSubTab === 'skins'
        ? 'bg-red-600 text-white shadow-lg shadow-red-600/30'
        : 'bg-white/4 hover:bg-white/8 text-neutral-400 hover:text-white border border-white/6'}"
    >
      <Palette class="w-3.5 h-3.5" />
      <span>Skin Presets</span>
    </button>

    <button
      type="button"
      onclick={() => (activeSubTab = "capes")}
      class="px-4 py-2 rounded-xl text-xs font-bold uppercase tracking-wider flex items-center gap-2 transition-all cursor-pointer {activeSubTab === 'capes'
        ? 'bg-red-600 text-white shadow-lg shadow-red-600/30'
        : 'bg-white/4 hover:bg-white/8 text-neutral-400 hover:text-white border border-white/6'}"
    >
      <Crown class="w-3.5 h-3.5" />
      <span>Capes Collection</span>
    </button>

    <button
      type="button"
      onclick={() => (activeSubTab = "stealer")}
      class="px-4 py-2 rounded-xl text-xs font-bold uppercase tracking-wider flex items-center gap-2 transition-all cursor-pointer {activeSubTab === 'stealer'
        ? 'bg-red-600 text-white shadow-lg shadow-red-600/30'
        : 'bg-white/4 hover:bg-white/8 text-neutral-400 hover:text-white border border-white/6'}"
    >
      <Search class="w-3.5 h-3.5" />
      <span>Player Skin Lookup</span>
    </button>
  </div>

  <!-- Main Grid Layout -->
  <div class="grid grid-cols-1 lg:grid-cols-12 gap-8 items-start">
    <!-- Left: 3D Stage & Rig Controls (5 Cols) -->
    <div class="lg:col-span-5 flex flex-col items-center">
      <div
        ondragover={handleDragOver}
        ondragleave={handleDragLeave}
        ondrop={handleDrop}
        role="region"
        aria-label="3D Skin Viewer Stage"
        class="relative rounded-3xl overflow-hidden border {isDraggingFile
          ? 'border-red-500 ring-2 ring-red-500/50 bg-red-500/10'
          : 'border-white/10 bg-gradient-to-b from-[#141724]/90 via-[#0d0e17] to-[#07080d]'} p-6 shadow-2xl flex flex-col items-center w-full transition-all"
      >
        <!-- Atmospheric Studio Lighting Ring -->
        <div class="absolute -top-16 -left-16 w-52 h-52 bg-red-600/15 rounded-full blur-3xl pointer-events-none"></div>
        <div class="absolute -bottom-16 -right-16 w-52 h-52 bg-cyan-600/10 rounded-full blur-3xl pointer-events-none"></div>

        <!-- 3D Canvas Stage -->
        <div class="relative w-full flex items-center justify-center">
          <canvas
            bind:this={canvasEl}
            class="rounded-2xl cursor-grab active:cursor-grabbing shadow-2xl select-none"
          ></canvas>

          {#if launcher.isLoadingSkin}
            <div class="absolute inset-0 bg-black/70 backdrop-blur-sm rounded-2xl flex flex-col items-center justify-center gap-2.5 text-xs text-white">
              <Loader2 class="w-7 h-7 text-red-500 animate-spin" />
              <span class="font-bold tracking-wide">Compiling 3D Textures...</span>
            </div>
          {/if}

          <!-- Drag & Drop Overlay Prompt -->
          {#if isDraggingFile}
            <div class="absolute inset-0 bg-red-950/80 backdrop-blur-sm rounded-2xl border-2 border-dashed border-red-500 flex flex-col items-center justify-center gap-2 text-white">
              <Upload class="w-8 h-8 text-red-400 animate-bounce" />
              <span class="text-xs font-black uppercase tracking-wider">Drop Skin or Cape PNG here</span>
            </div>
          {/if}
        </div>

        <!-- Camera Angle Preset Pills -->
        <div class="mt-4 w-full flex items-center justify-between gap-1 bg-white/4 p-1 rounded-xl border border-white/6">
          <button
            type="button"
            onclick={() => setCameraAngle(0)}
            class="flex-1 py-1 rounded-lg text-[11px] font-bold text-neutral-300 hover:text-white hover:bg-white/10 transition-all cursor-pointer"
            title="Front View"
          >
            Front
          </button>
          <button
            type="button"
            onclick={() => setCameraAngle(Math.PI)}
            class="flex-1 py-1 rounded-lg text-[11px] font-bold text-neutral-300 hover:text-white hover:bg-white/10 transition-all cursor-pointer"
            title="Back View"
          >
            Back
          </button>
          <button
            type="button"
            onclick={() => setCameraAngle(-Math.PI / 2)}
            class="flex-1 py-1 rounded-lg text-[11px] font-bold text-neutral-300 hover:text-white hover:bg-white/10 transition-all cursor-pointer"
            title="Left Side"
          >
            Left
          </button>
          <button
            type="button"
            onclick={() => setCameraAngle(Math.PI / 2)}
            class="flex-1 py-1 rounded-lg text-[11px] font-bold text-neutral-300 hover:text-white hover:bg-white/10 transition-all cursor-pointer"
            title="Right Side"
          >
            Right
          </button>
          <button
            type="button"
            onclick={toggleAutoRotate}
            class="px-2 py-1 rounded-lg text-[11px] font-bold transition-all cursor-pointer {isAutoRotating
              ? 'bg-cyan-600 text-white shadow-md'
              : 'text-neutral-400 hover:text-white'}"
            title="Auto Rotate 360"
          >
            Spin
          </button>
          <button
            type="button"
            onclick={initViewer}
            class="px-2 py-1 rounded-lg text-[11px] font-bold text-neutral-400 hover:text-white transition-all cursor-pointer"
            title="Reset Camera Angle"
          >
            <RotateCcw class="w-3 h-3" />
          </button>
        </div>

        <!-- Pose Animation & Arm Thickness -->
        <div class="mt-3 w-full space-y-2.5">
          <!-- Animation Modes -->
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
              onclick={() => applyAnimation("fly")}
              class="flex-1 py-1.5 rounded-lg text-xs font-bold transition-all cursor-pointer {animationMode === 'fly'
                ? 'bg-red-600 text-white shadow-md'
                : 'text-neutral-400 hover:text-white'}"
            >
              Fly
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
            {#if animationMode !== "idle"}
              <button
                type="button"
                onclick={togglePause}
                class="px-2.5 py-1.5 rounded-lg text-xs font-bold bg-white/6 hover:bg-white/12 text-white transition-all cursor-pointer"
                title={isPaused ? "Resume Animation" : "Pause Animation"}
              >
                {#if isPaused}
                  <Play class="w-3.5 h-3.5 text-emerald-400" />
                {:else}
                  <Pause class="w-3.5 h-3.5 text-amber-400" />
                {/if}
              </button>
            {/if}
          </div>

          <!-- Arm Model Type + Cape Visibility -->
          <div class="flex items-center justify-between gap-2 pt-1">
            <div class="flex items-center gap-1 bg-white/4 p-1 rounded-xl border border-white/6 flex-1">
              <button
                type="button"
                onclick={() => setModelType("classic")}
                class="flex-1 py-1 rounded-lg text-xs font-bold transition-all cursor-pointer {modelType === 'classic'
                  ? 'bg-white/15 text-white border border-white/20'
                  : 'text-neutral-400 hover:text-white'}"
              >
                Classic 4px
              </button>
              <button
                type="button"
                onclick={() => setModelType("slim")}
                class="flex-1 py-1 rounded-lg text-xs font-bold transition-all cursor-pointer {modelType === 'slim'
                  ? 'bg-white/15 text-white border border-white/20'
                  : 'text-neutral-400 hover:text-white'}"
              >
                Slim 3px
              </button>
            </div>

            <button
              type="button"
              onclick={toggleCapeVisibility}
              class="px-3 py-2 rounded-xl text-xs font-bold flex items-center gap-1.5 transition-all cursor-pointer {showCapeOnModel
                ? 'bg-purple-600/20 text-purple-300 border border-purple-500/30'
                : 'bg-white/5 text-neutral-400 border border-white/8'}"
              title="Toggle Cape Display on 3D Stage"
            >
              {#if showCapeOnModel}
                <Eye class="w-3.5 h-3.5 text-purple-400" />
                <span>Cape</span>
              {:else}
                <EyeOff class="w-3.5 h-3.5 text-neutral-500" />
                <span>Cape</span>
              {/if}
            </button>
          </div>
        </div>
      </div>
    </div>

    <!-- Right: Content Panes (7 Cols) -->
    <div class="lg:col-span-7 space-y-5">
      <!-- SUB-TAB 1: STUDIO & ACTIVE SKIN -->
      {#if activeSubTab === "studio"}
        <!-- Active Custom Skin & Cape Uploader Card -->
        <div class="p-6 rounded-2xl bg-[#0e1017]/90 border border-white/8 space-y-4">
          <div class="flex items-center justify-between pb-3 border-b border-white/8">
            <div class="flex items-center gap-2 text-white font-bold text-sm uppercase tracking-wider">
              <Upload class="w-4 h-4 text-red-500" />
              <span>Apply Custom Textures</span>
            </div>
            <button
              type="button"
              onclick={downloadCurrentSkin}
              class="text-xs text-neutral-400 hover:text-white flex items-center gap-1.5 transition-colors cursor-pointer"
              title="Download your active skin texture PNG"
            >
              <Download class="w-3.5 h-3.5" />
              <span>Save PNG</span>
            </button>
          </div>

          <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
            <!-- Skin Uploader -->
            <button
              type="button"
              onclick={() => launcher.pickAndApplySkin()}
              class="p-5 rounded-2xl bg-white/2 hover:bg-red-500/10 border border-white/8 hover:border-red-500/30 text-left transition-all cursor-pointer group flex flex-col justify-between"
            >
              <div>
                <div class="w-10 h-10 rounded-xl bg-red-500/10 border border-red-500/20 flex items-center justify-center text-red-400 group-hover:scale-105 transition-transform mb-3">
                  <Shirt class="w-5 h-5" />
                </div>
                <div class="text-sm font-bold text-white group-hover:text-red-400 transition-colors">
                  Choose Skin File
                </div>
                <p class="text-xs text-neutral-400 mt-1 leading-relaxed">
                  Select any standard 64x64 PNG Minecraft skin. Drag & drop directly onto the 3D model also supported.
                </p>
              </div>

              <div class="mt-4 flex items-center gap-1.5 text-xs text-red-400 font-bold uppercase tracking-wider">
                <span>Browse File</span>
                <ArrowUpRight class="w-3.5 h-3.5" />
              </div>
            </button>

            <!-- Cape Uploader -->
            <button
              type="button"
              onclick={() => launcher.pickAndApplyCape()}
              class="p-5 rounded-2xl bg-white/2 hover:bg-purple-500/10 border border-white/8 hover:border-purple-500/30 text-left transition-all cursor-pointer group flex flex-col justify-between"
            >
              <div>
                <div class="w-10 h-10 rounded-xl bg-purple-500/10 border border-purple-500/20 flex items-center justify-center text-purple-400 group-hover:scale-105 transition-transform mb-3">
                  <Crown class="w-5 h-5" />
                </div>
                <div class="text-sm font-bold text-white group-hover:text-purple-400 transition-colors">
                  Choose Cape File
                </div>
                <p class="text-xs text-neutral-400 mt-1 leading-relaxed">
                  Select standard 64x32 PNG cape cosmetics. Injected natively into game launch resource packs.
                </p>
              </div>

              <div class="mt-4 flex items-center gap-1.5 text-xs text-purple-400 font-bold uppercase tracking-wider">
                <span>Browse File</span>
                <ArrowUpRight class="w-3.5 h-3.5" />
              </div>
            </button>
          </div>

          <!-- Quick Reset Toolbar -->
          <div class="flex items-center gap-3 pt-2">
            <button
              type="button"
              onclick={() => launcher.resetPlayerSkin()}
              class="flex-1 py-2 px-3 rounded-xl bg-white/4 hover:bg-white/8 border border-white/8 text-neutral-300 hover:text-white text-xs font-semibold flex items-center justify-center gap-2 transition-colors cursor-pointer"
            >
              <RotateCcw class="w-3.5 h-3.5" />
              <span>Reset Skin to Steve</span>
            </button>

            {#if launcher.activeCapeDataUrl}
              <button
                type="button"
                onclick={() => launcher.resetPlayerCape()}
                class="flex-1 py-2 px-3 rounded-xl bg-rose-500/10 hover:bg-rose-500/20 border border-rose-500/20 text-rose-300 text-xs font-semibold flex items-center justify-center gap-2 transition-colors cursor-pointer"
              >
                <Trash2 class="w-3.5 h-3.5" />
                <span>Remove Active Cape</span>
              </button>
            {/if}
          </div>
        </div>

        <!-- Technical Diagnostics Card -->
        <div class="p-5 rounded-2xl bg-[#0e1017]/90 border border-white/8 space-y-3">
          <div class="text-xs text-neutral-400 font-bold uppercase tracking-wider">
            Active Texture Specifications
          </div>
          <div class="grid grid-cols-3 gap-3">
            <div class="p-3 rounded-xl bg-white/2 border border-white/6">
              <span class="text-[10px] text-neutral-400 block uppercase font-bold">Skin Format</span>
              <span class="text-xs font-mono font-bold text-emerald-400">64x64 RGBA</span>
            </div>
            <div class="p-3 rounded-xl bg-white/2 border border-white/6">
              <span class="text-[10px] text-neutral-400 block uppercase font-bold">Model Rig</span>
              <span class="text-xs font-mono font-bold text-white capitalize">{modelType} Arm</span>
            </div>
            <div class="p-3 rounded-xl bg-white/2 border border-white/6">
              <span class="text-[10px] text-neutral-400 block uppercase font-bold">Equipped Cape</span>
              <span class="text-xs font-mono font-bold {launcher.activeCapeDataUrl ? 'text-purple-400' : 'text-neutral-500'}">
                {launcher.activeCapeDataUrl ? "Active" : "None"}
              </span>
            </div>
          </div>
        </div>

      <!-- SUB-TAB 2: SKIN PRESETS -->
      {:else if activeSubTab === "skins"}
        <div class="p-6 rounded-2xl bg-[#0e1017]/90 border border-white/8 space-y-4">
          <div>
            <h3 class="text-sm font-bold text-white uppercase tracking-wider flex items-center gap-2">
              <Palette class="w-4 h-4 text-red-400" />
              <span>Curated Skin Presets</span>
            </h3>
            <p class="text-xs text-neutral-400 mt-0.5">
              Select any pre-configured high-res skin to apply to your current profile in 1 click.
            </p>
          </div>

          <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
            {#each SKIN_PRESETS as preset}
              <div class="p-5 rounded-2xl bg-white/2 border border-white/8 hover:border-white/16 transition-all flex flex-col justify-between space-y-4">
                <div>
                  <div class="flex items-center justify-between mb-2">
                    <span class="px-2 py-0.5 rounded text-[10px] font-black uppercase bg-gradient-to-r {preset.color} border">
                      {preset.tag}
                    </span>
                    <span class="text-[10px] text-neutral-400 font-mono capitalize">{preset.model}</span>
                  </div>
                  <h4 class="text-base font-bold text-white">{preset.name}</h4>
                </div>

                <button
                  type="button"
                  onclick={() => {
                    setModelType(preset.model);
                    launcher.applySkinBase64(preset.dataUrl);
                  }}
                  class="w-full py-2.5 rounded-xl bg-red-600 hover:bg-red-500 text-white font-bold text-xs uppercase tracking-wider flex items-center justify-center gap-2 transition-all cursor-pointer shadow-md shadow-red-600/30"
                >
                  <Check class="w-3.5 h-3.5" />
                  <span>Equip Skin</span>
                </button>
              </div>
            {/each}
          </div>
        </div>

      <!-- SUB-TAB 3: CAPES COLLECTION -->
      {:else if activeSubTab === "capes"}
        <div class="p-6 rounded-2xl bg-[#0e1017]/90 border border-white/8 space-y-4">
          <div>
            <h3 class="text-sm font-bold text-white uppercase tracking-wider flex items-center gap-2">
              <Crown class="w-4 h-4 text-purple-400" />
              <span>Exclusive Capes Collection</span>
            </h3>
            <p class="text-xs text-neutral-400 mt-0.5">
              Equip animated cosmetics and badges visible in third-person view and inventory.
            </p>
          </div>

          <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
            {#each CAPE_PRESETS as cape}
              <div class="p-5 rounded-2xl bg-white/2 border border-white/8 hover:border-white/16 transition-all flex flex-col justify-between space-y-4">
                <div>
                  <div class="flex items-center justify-between mb-2">
                    <span class="px-2 py-0.5 rounded text-[10px] font-black uppercase bg-gradient-to-r {cape.color} border">
                      {cape.rarity}
                    </span>
                    <span class="text-[10px] text-neutral-400 font-mono">64x32</span>
                  </div>
                  <h4 class="text-base font-bold text-white">{cape.name}</h4>
                  <p class="text-[11px] text-neutral-400 mt-1">{cape.tag}</p>
                </div>

                <button
                  type="button"
                  onclick={() => launcher.applyCapeBase64(cape.dataUrl)}
                  class="w-full py-2.5 rounded-xl bg-purple-600 hover:bg-purple-500 text-white font-bold text-xs uppercase tracking-wider flex items-center justify-center gap-2 transition-all cursor-pointer shadow-md shadow-purple-600/30"
                >
                  <Check class="w-3.5 h-3.5" />
                  <span>Equip Cape</span>
                </button>
              </div>
            {/each}
          </div>
        </div>

      <!-- SUB-TAB 4: PLAYER SKIN LOOKUP / STEALER -->
      {:else if activeSubTab === "stealer"}
        <div class="p-6 rounded-2xl bg-[#0e1017]/90 border border-white/8 space-y-5">
          <div>
            <h3 class="text-sm font-bold text-white uppercase tracking-wider flex items-center gap-2">
              <Search class="w-4 h-4 text-cyan-400" />
              <span>Player Skin Lookup & Stealer</span>
            </h3>
            <p class="text-xs text-neutral-400 mt-0.5">
              Fetch the official skin texture of any Minecraft Java player and apply it immediately to your profile.
            </p>
          </div>

          <div class="space-y-3">
            <div class="flex items-center gap-2">
              <input
                type="text"
                bind:value={searchUsername}
                placeholder="Enter Minecraft Java username (e.g. Notch, Technoblade)..."
                class="flex-1 px-4 py-2.5 rounded-xl bg-white/4 border border-white/10 text-xs font-mono text-white outline-none focus:border-cyan-500/50 transition-colors"
                onkeydown={(e) => {
                  if (e.key === "Enter" && searchUsername.trim()) {
                    launcher.importSkinByUsername(searchUsername.trim());
                  }
                }}
              />
              <button
                type="button"
                disabled={!searchUsername.trim() || launcher.isLoadingSkin}
                onclick={() => launcher.importSkinByUsername(searchUsername.trim())}
                class="px-5 py-2.5 rounded-xl bg-cyan-600 hover:bg-cyan-500 disabled:opacity-50 text-white font-bold text-xs uppercase tracking-wider flex items-center gap-2 transition-all cursor-pointer shadow-md shadow-cyan-600/30 shrink-0"
              >
                {#if launcher.isLoadingSkin}
                  <Loader2 class="w-4 h-4 animate-spin" />
                {:else}
                  <Download class="w-4 h-4" />
                {/if}
                <span>Fetch & Equip</span>
              </button>
            </div>

            <!-- Quick Creator Shortcuts -->
            <div class="pt-2">
              <span class="text-[11px] text-neutral-400 font-bold uppercase tracking-wider block mb-2">
                Popular Creator Skins:
              </span>
              <div class="flex flex-wrap gap-2">
                {#each ["Notch", "Technoblade", "MumboJumbo", "Grian", "DanTDM", "CaptainSparklez"] as creator}
                  <button
                    type="button"
                    onclick={() => launcher.importSkinByUsername(creator)}
                    class="px-3 py-1.5 rounded-xl bg-white/4 hover:bg-cyan-500/20 hover:border-cyan-500/30 border border-white/8 text-xs font-semibold text-neutral-300 hover:text-white transition-all cursor-pointer flex items-center gap-1.5"
                  >
                    <span>{creator}</span>
                  </button>
                {/each}
              </div>
            </div>
          </div>
        </div>
      {/if}
    </div>
  </div>
</div>
