<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { launcher } from "../../state/launcher.svelte";
  import * as skinview3d from "skinview3d";
  import { 
    User, 
    Upload, 
    Shield, 
    RotateCcw, 
    Sparkles, 
    Info, 
    Layers 
  } from "lucide-svelte";

  let canvasEl: HTMLCanvasElement | null = $state(null);
  let viewer: skinview3d.SkinViewer | null = $state(null);
  let isWalking = $state(true);

  function getAvatarSkinUrl() {
    if (launcher.activeAccount?.id && launcher.activeAccount.account_type === "microsoft") {
      return `https://crafthead.net/skin/${launcher.activeAccount.id}`;
    }
    // Steve default texture
    return "https://textures.minecraft.net/texture/3b60a1f6d2719b0d24e1644781df5a4628c64ee32f6b864f16a0daeaad6ad9e";
  }

  function initViewer() {
    if (!canvasEl) return;
    if (viewer) viewer.dispose();

    try {
      viewer = new skinview3d.SkinViewer({
        canvas: canvasEl,
        width: 320,
        height: 440,
        skin: getAvatarSkinUrl()
      });

      if (isWalking) {
        viewer.animation = new skinview3d.WalkingAnimation();
        viewer.animation.speed = 0.7;
      }
    } catch (e) {
      console.warn("Could not initialize 3D skin viewer:", e);
    }
  }

  function toggleAnimation() {
    if (!viewer) return;
    isWalking = !isWalking;
    if (isWalking) {
      viewer.animation = new skinview3d.WalkingAnimation();
      viewer.animation.speed = 0.7;
    } else {
      viewer.animation = null;
    }
  }

  onMount(() => {
    initViewer();
  });

  onDestroy(() => {
    if (viewer) viewer.dispose();
  });

  async function handleApplySkin() {
    await launcher.pickAndApplySkin();
    initViewer();
  }

  async function handleApplyCape() {
    await launcher.pickAndApplyCape();
    initViewer();
  }
</script>

<div class="h-full overflow-y-auto p-8 space-y-6">
  <!-- Top Bar -->
  <div class="flex items-center justify-between pb-4 border-b border-white/6">
    <div>
      <h2 class="text-2xl font-black text-white uppercase tracking-tight flex items-center gap-3">
        <User class="w-6 h-6 text-red-500" />
        <span>3D Skins & Capes Studio</span>
      </h2>
      <p class="text-xs text-neutral-400 font-mono mt-1">
        Customize your in-game player character and cosmetic capes in real-time 3D.
      </p>
    </div>
  </div>

  <div class="grid grid-cols-1 md:grid-cols-12 gap-8 items-start">
    <!-- 3D Canvas Box -->
    <div class="md:col-span-6 flex flex-col items-center">
      <div class="relative rounded-3xl overflow-hidden border border-white/10 bg-gradient-to-b from-[#141724]/60 to-[#0b0c13] p-4 shadow-2xl flex flex-col items-center">
        <!-- Ambient Red Specular Flare -->
        <div class="absolute -top-12 -left-12 w-48 h-48 bg-red-600/10 rounded-full blur-2xl pointer-events-none"></div>

        <canvas 
          bind:this={canvasEl} 
          class="rounded-2xl cursor-grab active:cursor-grabbing"
        ></canvas>

        <div class="mt-4 flex items-center gap-3">
          <button
            onclick={toggleAnimation}
            class="px-4 py-1.5 rounded-xl bg-white/5 hover:bg-white/10 border border-white/8 text-xs font-mono text-neutral-300 font-semibold transition-all cursor-pointer"
          >
            {isWalking ? "Pause Animation" : "Play Animation"}
          </button>

          <button
            onclick={initViewer}
            title="Reset 3D camera angle"
            class="p-2 rounded-xl bg-white/5 hover:bg-white/10 border border-white/8 text-neutral-400 hover:text-white transition-all cursor-pointer"
          >
            <RotateCcw class="w-3.5 h-3.5" />
          </button>
        </div>
      </div>
    </div>

    <!-- Actions & Info Cards -->
    <div class="md:col-span-6 space-y-5">
      <!-- Active Account Tag -->
      <div class="p-5 rounded-2xl bg-[#0e1017]/90 border border-white/8 space-y-4">
        <h3 class="text-sm font-bold text-white uppercase tracking-wider font-mono flex items-center gap-2">
          <Shield class="w-4 h-4 text-red-400" />
          <span>Active Player Target</span>
        </h3>

        {#if launcher.activeAccount}
          <div class="flex items-center gap-3 p-3 rounded-xl bg-white/3 border border-white/6">
            <div class="w-10 h-10 rounded-lg bg-red-500/10 border border-red-500/20 flex items-center justify-center text-red-400 font-black font-mono">
              {launcher.activeAccount.username.substring(0, 2).toUpperCase()}
            </div>
            <div>
              <div class="text-sm font-bold text-white">{launcher.activeAccount.username}</div>
              <div class="text-[10px] font-mono text-neutral-400 uppercase">
                {launcher.activeAccount.account_type === "microsoft" ? "Microsoft Verified" : "Offline Local Profile"}
              </div>
            </div>
          </div>
        {:else}
          <div class="p-3 text-xs text-amber-400/90 font-mono bg-amber-500/10 border border-amber-500/20 rounded-xl">
            Please log in or add a player profile first to assign skins.
          </div>
        {/if}
      </div>

      <!-- Upload Actions -->
      <div class="p-5 rounded-2xl bg-[#0e1017]/90 border border-white/8 space-y-4">
        <h3 class="text-sm font-bold text-white uppercase tracking-wider font-mono flex items-center gap-2">
          <Upload class="w-4 h-4 text-emerald-400" />
          <span>Skin & Cape File Upload</span>
        </h3>

        <div class="grid grid-cols-2 gap-3">
          <button
            onclick={handleApplySkin}
            class="p-4 rounded-xl bg-white/4 hover:bg-red-500/15 border border-white/8 hover:border-red-500/30 text-left transition-all cursor-pointer group"
          >
            <div class="w-8 h-8 rounded-lg bg-white/6 flex items-center justify-center text-neutral-300 group-hover:text-red-400 mb-2">
              <Upload class="w-4 h-4" />
            </div>
            <div class="text-xs font-bold text-white group-hover:text-red-400">Apply Skin</div>
            <div class="text-[10px] font-mono text-neutral-500">64x64 PNG file</div>
          </button>

          <button
            onclick={handleApplyCape}
            class="p-4 rounded-xl bg-white/4 hover:bg-red-500/15 border border-white/8 hover:border-red-500/30 text-left transition-all cursor-pointer group"
          >
            <div class="w-8 h-8 rounded-lg bg-white/6 flex items-center justify-center text-neutral-300 group-hover:text-red-400 mb-2">
              <Layers class="w-4 h-4" />
            </div>
            <div class="text-xs font-bold text-white group-hover:text-red-400">Apply Cape</div>
            <div class="text-[10px] font-mono text-neutral-500">64x32 PNG file</div>
          </button>
        </div>

        <div class="p-3 rounded-xl bg-white/2 border border-white/4 flex items-start gap-2.5 text-[11px] font-mono text-neutral-400 leading-relaxed">
          <Info class="w-4 h-4 text-sky-400 shrink-0 mt-0.5" />
          <span>
            Offline and modded player skins are injected automatically into the game's virtual skin pack texture cache upon launch.
          </span>
        </div>
      </div>
    </div>
  </div>
</div>
