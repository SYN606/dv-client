<script lang="ts">
  import { launcher } from "../../state/launcher.svelte";
  import {
    Gamepad2,
    FolderKanban,
    Wrench,
    User,
    Settings,
    Terminal,
    Coffee,
    Cpu,
  } from "lucide-svelte";
</script>

<aside
  class="w-64 border-r border-white/8 bg-[#0a0c12]/95 backdrop-blur-2xl flex flex-col justify-between p-4 shrink-0 shadow-2xl z-20"
>
  <div class="space-y-6">
    <!-- Brand Crest Header -->
    <div class="flex items-center gap-3 px-2 pt-2">
      <div
        class="relative w-10 h-10 rounded-2xl bg-gradient-to-br from-red-500 via-rose-600 to-red-700 flex items-center justify-center shadow-lg shadow-red-500/25 group"
      >
        <div class="font-black text-lg tracking-tight text-white drop-shadow">
          DV
        </div>
        <span
          class="absolute -bottom-1 -right-1 w-3.5 h-3.5 rounded-full bg-emerald-500 border-2 border-[#0a0c12]"
        ></span>
      </div>
      <div>
        <h1 class="font-bold text-sm tracking-tight uppercase text-white">
          Digital Vigital
        </h1>
        <div
          class="flex items-center gap-1.5 text-xs text-neutral-400 font-medium"
        >
          <span>v{launcher.launcherInfo.version}</span>
          <span class="text-neutral-600">•</span>
          <span
            class="text-red-400 font-bold uppercase tracking-wider text-[10px]"
            >Tauri v2</span
          >
        </div>
      </div>
    </div>

    <!-- Navigation Tabs -->
    <nav class="space-y-1.5 font-sans">
      <button
        onclick={() => (launcher.activeTab = "play")}
        class="w-full flex items-center gap-3.5 px-3.5 py-3 rounded-xl text-xs font-bold uppercase tracking-wider transition-all cursor-pointer {launcher.activeTab ===
        'play'
          ? 'bg-red-500/15 text-red-400 border border-red-500/30 shadow-lg shadow-red-500/10'
          : 'text-neutral-300 hover:text-white hover:bg-white/5'}"
      >
        <Gamepad2
          class="w-4 h-4 {launcher.activeTab === 'play'
            ? 'text-red-400'
            : 'text-neutral-400'}"
        />
        <span>Launch & Play</span>
      </button>

      <button
        onclick={() => (launcher.activeTab = "instances")}
        class="w-full flex items-center justify-between px-3.5 py-3 rounded-xl text-xs font-bold uppercase tracking-wider transition-all cursor-pointer {launcher.activeTab ===
        'instances'
          ? 'bg-red-500/15 text-red-400 border border-red-500/30 shadow-lg shadow-red-500/10'
          : 'text-neutral-300 hover:text-white hover:bg-white/5'}"
      >
        <div class="flex items-center gap-3.5">
          <FolderKanban
            class="w-4 h-4 {launcher.activeTab === 'instances'
              ? 'text-red-400'
              : 'text-neutral-400'}"
          />
          <span>Profiles</span>
        </div>
        <span
          class="px-2 py-0.5 rounded-md bg-white/8 font-mono text-xs text-neutral-300 font-medium"
        >
          {launcher.instances.length}
        </span>
      </button>

      <button
        onclick={() => (launcher.activeTab = "mods")}
        class="w-full flex items-center justify-between px-3.5 py-3 rounded-xl text-xs font-bold uppercase tracking-wider transition-all cursor-pointer {launcher.activeTab ===
        'mods'
          ? 'bg-red-500/15 text-red-400 border border-red-500/30 shadow-lg shadow-red-500/10'
          : 'text-neutral-300 hover:text-white hover:bg-white/5'}"
      >
        <div class="flex items-center gap-3.5">
          <Wrench
            class="w-4 h-4 {launcher.activeTab === 'mods'
              ? 'text-red-400'
              : 'text-neutral-400'}"
          />
          <span>Mod Studio</span>
        </div>
        {#if launcher.installedMods.length > 0}
          <span
            class="px-2 py-0.5 rounded-md bg-red-500/20 text-red-400 font-mono text-xs font-bold"
          >
            {launcher.installedMods.length}
          </span>
        {/if}
      </button>

      <button
        onclick={() => (launcher.activeTab = "skins")}
        class="w-full flex items-center gap-3.5 px-3.5 py-3 rounded-xl text-xs font-bold uppercase tracking-wider transition-all cursor-pointer {launcher.activeTab ===
        'skins'
          ? 'bg-red-500/15 text-red-400 border border-red-500/30 shadow-lg shadow-red-500/10'
          : 'text-neutral-300 hover:text-white hover:bg-white/5'}"
      >
        <User
          class="w-4 h-4 {launcher.activeTab === 'skins'
            ? 'text-red-400'
            : 'text-neutral-400'}"
        />
        <span>3D Skins & Capes</span>
      </button>

      <button
        onclick={() => (launcher.activeTab = "settings")}
        class="w-full flex items-center gap-3.5 px-3.5 py-3 rounded-xl text-xs font-bold uppercase tracking-wider transition-all cursor-pointer {launcher.activeTab ===
        'settings'
          ? 'bg-red-500/15 text-red-400 border border-red-500/30 shadow-lg shadow-red-500/10'
          : 'text-neutral-300 hover:text-white hover:bg-white/5'}"
      >
        <Settings
          class="w-4 h-4 {launcher.activeTab === 'settings'
            ? 'text-red-400'
            : 'text-neutral-400'}"
        />
        <span>Java & Engine</span>
      </button>

      <button
        onclick={() => (launcher.activeTab = "console")}
        class="w-full flex items-center justify-between px-3.5 py-3 rounded-xl text-xs font-bold uppercase tracking-wider transition-all cursor-pointer {launcher.activeTab ===
        'console'
          ? 'bg-red-500/15 text-red-400 border border-red-500/30 shadow-lg shadow-red-500/10'
          : 'text-neutral-300 hover:text-white hover:bg-white/5'}"
      >
        <div class="flex items-center gap-3.5">
          <Terminal
            class="w-4 h-4 {launcher.activeTab === 'console'
              ? 'text-red-400'
              : 'text-neutral-400'}"
          />
          <span>Live Console</span>
        </div>
        {#if launcher.gameStatus === "running"}
          <span class="w-2.5 h-2.5 rounded-full bg-emerald-400 animate-ping"
          ></span>
        {/if}
      </button>
    </nav>
  </div>

  <!-- Quick Status Hardware Footer -->
  <div
    class="p-3.5 rounded-2xl bg-white/4 border border-white/8 space-y-2.5 text-xs text-neutral-300"
  >
    <div class="flex items-center justify-between">
      <span class="flex items-center gap-2 text-neutral-400"
        ><Coffee class="w-4 h-4 text-amber-400" /> Java</span
      >
      <span class="text-neutral-100 font-bold truncate max-w-[100px]">
        {launcher.detectedJavas.find((j) => j.is_recommended)?.display_name ||
          launcher.detectedJavas[0]?.display_name ||
          "Auto"}
      </span>
    </div>
    <div class="flex items-center justify-between">
      <span class="flex items-center gap-2 text-neutral-400"
        ><Cpu class="w-4 h-4 text-sky-400" /> GPU</span
      >
      <span class="text-neutral-100 font-bold truncate max-w-[100px]">
        {launcher.detectedGpus
          .find((g) => g.is_discrete)
          ?.name.replace(/NVIDIA|GeForce|AMD|Radeon/gi, "")
          .trim() || "Dedicated"}
      </span>
    </div>
    <div
      class="flex items-center justify-between pt-2 border-t border-white/8 text-xs"
    >
      <span class="text-neutral-400">Status</span>
      {#if launcher.gameStatus === "running"}
        <span class="text-emerald-400 font-bold flex items-center gap-1.5">
          <span class="w-2 h-2 rounded-full bg-emerald-400 animate-pulse"
          ></span>
          Running
        </span>
      {:else if launcher.gameStatus === "launching" || launcher.gameStatus === "downloading"}
        <span class="text-amber-400 font-bold flex items-center gap-1.5">
          <span class="w-2 h-2 rounded-full bg-amber-400 animate-spin"></span>
          Starting
        </span>
      {:else}
        <span class="text-neutral-400 flex items-center gap-1.5">
          <span class="w-2 h-2 rounded-full bg-neutral-600"></span>
          Standby
        </span>
      {/if}
    </div>
  </div>
</aside>
