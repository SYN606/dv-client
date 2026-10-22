<script lang="ts">
  import { launcher } from "../../state/launcher.svelte";
  import { 
    Play, 
    Square, 
    Sparkles, 
    Zap, 
    FolderOpen, 
    Wrench, 
    CheckCircle2, 
    HardDrive, 
    Coffee, 
    Cpu, 
    Server,
    Copy,
    ArrowUpRight
  } from "lucide-svelte";

  let copiedIp = $state(false);

  function copyServerIp() {
    navigator.clipboard.writeText("play.digitalvigital.net");
    copiedIp = true;
    launcher.showToast("Server IP copied to clipboard: play.digitalvigital.net");
    setTimeout(() => { copiedIp = false; }, 2500);
  }

  function getLoaderColor(loader: string = "") {
    const l = loader.toLowerCase();
    if (l.includes("fabric")) return "text-purple-400 border-purple-500/30 bg-purple-500/10";
    if (l.includes("neoforge")) return "text-orange-400 border-orange-500/30 bg-orange-500/10";
    if (l.includes("forge")) return "text-amber-400 border-amber-500/30 bg-amber-500/10";
    return "text-neutral-400 border-neutral-500/30 bg-neutral-500/10";
  }
</script>

<div class="h-full overflow-y-auto p-8 space-y-6">
  <!-- Hero Card inspired by SKlauncher & FastClient -->
  <div class="relative rounded-3xl overflow-hidden border border-white/10 bg-gradient-to-br from-[#12141c] via-[#0c0d14] to-[#18090d] shadow-2xl p-8 flex flex-col justify-between min-h-[290px]">
    <!-- Ambient Red Specular Flare -->
    <div class="absolute -top-24 -right-24 w-96 h-96 bg-red-600/15 rounded-full blur-3xl pointer-events-none"></div>
    <div class="absolute bottom-0 right-1/4 w-80 h-40 bg-purple-600/10 rounded-full blur-3xl pointer-events-none"></div>

    <!-- Top Info Row in Hero -->
    <div class="relative z-10 flex items-start justify-between">
      <div>
        <div class="inline-flex items-center gap-2 px-3 py-1 rounded-full bg-red-500/10 border border-red-500/20 text-red-400 text-xs font-mono font-bold tracking-wider uppercase mb-3">
          <Zap class="w-3.5 h-3.5 fill-red-400" />
          <span>Ultra-Low Latency Engine</span>
        </div>
        <h2 class="text-3xl font-black text-white tracking-tight uppercase font-sans">
          {launcher.activeInstance ? launcher.activeInstance.name : "Select or Create Profile"}
        </h2>
        <div class="flex items-center gap-3 mt-2 text-xs font-mono text-neutral-400">
          {#if launcher.activeInstance}
            <span class="px-2 py-0.5 rounded border {getLoaderColor(launcher.activeInstance.mod_loader)} font-bold uppercase">
              {launcher.activeInstance.mod_loader}
            </span>
            <span>•</span>
            <span class="text-neutral-300">Minecraft {launcher.activeInstance.mc_version}</span>
            <span>•</span>
            <span class="text-neutral-400">{launcher.installedMods.length} mods installed</span>
          {:else}
            <span class="text-neutral-500">Pick an active profile to begin playing</span>
          {/if}
        </div>
      </div>

      <!-- Quick Action Pill for Mods & Folder -->
      {#if launcher.activeInstance}
        <div class="flex items-center gap-2">
          <button
            onclick={() => launcher.openModsFolder()}
            title="Open instance folder"
            class="px-3 py-2 rounded-xl bg-white/5 hover:bg-white/10 border border-white/8 text-neutral-300 hover:text-white text-xs font-semibold flex items-center gap-2 transition-all cursor-pointer"
          >
            <FolderOpen class="w-3.5 h-3.5" />
            <span>Folder</span>
          </button>
          <button
            onclick={() => launcher.activeTab = "mods"}
            title="Open Mod Studio"
            class="px-3 py-2 rounded-xl bg-white/5 hover:bg-white/10 border border-white/8 text-neutral-300 hover:text-white text-xs font-semibold flex items-center gap-2 transition-all cursor-pointer"
          >
            <Wrench class="w-3.5 h-3.5" />
            <span>Mods</span>
          </button>
        </div>
      {/if}
    </div>

    <!-- Bottom Launch Area -->
    <div class="relative z-10 pt-6 flex flex-col md:flex-row items-center justify-between gap-6">
      <!-- Status & Download Progress -->
      <div class="w-full md:w-auto flex-1 space-y-2">
        <div class="flex items-center gap-2.5 text-xs font-mono text-neutral-300">
          {#if launcher.gameStatus === "running"}
            <span class="w-2.5 h-2.5 rounded-full bg-emerald-400 animate-pulse"></span>
          {:else if launcher.gameStatus === "launching" || launcher.gameStatus === "downloading"}
            <span class="w-2.5 h-2.5 rounded-full bg-amber-400 animate-spin"></span>
          {:else}
            <span class="w-2.5 h-2.5 rounded-full bg-neutral-600"></span>
          {/if}
          <span class="font-semibold">{launcher.launchStatusText}</span>
        </div>

        {#if launcher.gameStatus === "downloading"}
          <div class="w-full max-w-md h-2 rounded-full bg-white/5 border border-white/10 overflow-hidden relative">
            <div 
              class="h-full bg-gradient-to-r from-red-600 to-rose-400 rounded-full transition-all duration-200"
              style="width: {launcher.downloadProgress.percent}%"
            ></div>
          </div>
          <div class="flex justify-between max-w-md text-[10px] font-mono text-neutral-500">
            <span>{launcher.downloadProgress.current} / {launcher.downloadProgress.total} items</span>
            <span>{launcher.downloadProgress.percent.toFixed(1)}%</span>
          </div>
        {/if}
      </div>

      <!-- Main Giant Play / Stop Button -->
      <button
        onclick={() => launcher.launchOrStop()}
        disabled={launcher.gameStatus === "launching"}
        class="w-full md:w-auto px-10 py-4 rounded-2xl font-black uppercase tracking-wider text-sm flex items-center justify-center gap-3 shadow-2xl transition-all cursor-pointer duration-200 active:scale-95 disabled:opacity-50 disabled:cursor-not-allowed {
          launcher.gameStatus === 'running' 
            ? 'bg-rose-600 hover:bg-rose-500 text-white shadow-rose-600/30 border border-rose-400/30' 
            : launcher.gameStatus === 'downloading' || launcher.gameStatus === 'launching'
            ? 'bg-amber-600 text-white cursor-wait border border-amber-400/30'
            : 'bg-gradient-to-r from-red-600 to-rose-600 hover:from-red-500 hover:to-rose-500 text-white shadow-red-600/40 hover:shadow-red-500/50 hover:scale-[1.02] border border-red-400/30'
        }"
      >
        {#if launcher.gameStatus === "running"}
          <Square class="w-5 h-5 fill-white" />
          <span>STOP MINECRAFT</span>
        {:else if launcher.gameStatus === "launching"}
          <div class="w-5 h-5 border-2 border-white/30 border-t-white rounded-full animate-spin"></div>
          <span>LAUNCHING...</span>
        {:else if launcher.gameStatus === "downloading"}
          <div class="w-5 h-5 border-2 border-white/30 border-t-white rounded-full animate-spin"></div>
          <span>DOWNLOADING...</span>
        {:else}
          <Play class="w-5 h-5 fill-white" />
          <span>PLAY MINECRAFT</span>
        {/if}
      </button>
    </div>
  </div>

  <!-- Quick Feature Grid (Inspired by SKlauncher & FastClient) -->
  <div class="grid grid-cols-1 md:grid-cols-3 gap-6">
    <!-- 1-Click FPS Optimizer Pack -->
    <div class="p-5 rounded-2xl bg-[#0e1017]/90 border border-white/8 hover:border-white/15 transition-all flex flex-col justify-between group">
      <div>
        <div class="flex items-center justify-between mb-3">
          <div class="w-10 h-10 rounded-xl bg-red-500/10 border border-red-500/20 flex items-center justify-center text-red-400 group-hover:scale-110 transition-transform">
            <Sparkles class="w-5 h-5" />
          </div>
          <span class="text-[10px] font-mono font-bold uppercase px-2 py-0.5 rounded bg-red-500/15 text-red-400">
            FastClient Spec
          </span>
        </div>
        <h3 class="font-bold text-sm text-white">Essential FPS Optimizer</h3>
        <p class="text-xs text-neutral-400 mt-1 leading-relaxed">
          Installs Sodium, Lithium, and FerriteCore for 3x higher FPS and stutter-free chunk loading.
        </p>
      </div>

      <button
        onclick={() => launcher.installEssentialFpsPack()}
        disabled={!launcher.activeInstance || launcher.installingModId !== ""}
        class="mt-4 w-full py-2.5 rounded-xl bg-white/5 hover:bg-red-500/20 hover:border-red-500/30 border border-white/8 text-neutral-200 hover:text-red-400 text-xs font-bold font-mono uppercase tracking-wider flex items-center justify-center gap-2 transition-all cursor-pointer disabled:opacity-50"
      >
        {#if launcher.installingModId}
          <div class="w-3.5 h-3.5 border-2 border-white/30 border-t-white rounded-full animate-spin"></div>
          <span>INSTALLING...</span>
        {:else}
          <Zap class="w-3.5 h-3.5" />
          <span>1-CLICK FPS BOOST</span>
        {/if}
      </button>
    </div>

    <!-- Official Community Server Card -->
    <div class="p-5 rounded-2xl bg-[#0e1017]/90 border border-white/8 hover:border-white/15 transition-all flex flex-col justify-between group">
      <div>
        <div class="flex items-center justify-between mb-3">
          <div class="w-10 h-10 rounded-xl bg-emerald-500/10 border border-emerald-500/20 flex items-center justify-center text-emerald-400 group-hover:scale-110 transition-transform">
            <Server class="w-5 h-5" />
          </div>
          <span class="text-[10px] font-mono font-bold uppercase px-2 py-0.5 rounded bg-emerald-500/15 text-emerald-400 flex items-center gap-1">
            <span class="w-1.5 h-1.5 rounded-full bg-emerald-400 animate-pulse"></span>
            Online
          </span>
        </div>
        <h3 class="font-bold text-sm text-white">Digital Vigital Network</h3>
        <p class="text-xs text-neutral-400 mt-1 leading-relaxed">
          Join the high-tickrate official multiplayer SMP & PvP arena server.
        </p>
      </div>

      <button
        onclick={copyServerIp}
        class="mt-4 w-full py-2.5 rounded-xl bg-white/5 hover:bg-emerald-500/20 hover:border-emerald-500/30 border border-white/8 text-neutral-200 hover:text-emerald-400 text-xs font-bold font-mono uppercase tracking-wider flex items-center justify-center gap-2 transition-all cursor-pointer"
      >
        {#if copiedIp}
          <CheckCircle2 class="w-3.5 h-3.5 text-emerald-400" />
          <span>COPIED IP!</span>
        {:else}
          <Copy class="w-3.5 h-3.5" />
          <span>play.digitalvigital.net</span>
        {/if}
      </button>
    </div>

    <!-- Active Hardware & Tuning Summary -->
    <div class="p-5 rounded-2xl bg-[#0e1017]/90 border border-white/8 hover:border-white/15 transition-all flex flex-col justify-between">
      <div>
        <div class="flex items-center justify-between mb-3">
          <div class="w-10 h-10 rounded-xl bg-purple-500/10 border border-purple-500/20 flex items-center justify-center text-purple-400">
            <HardDrive class="w-5 h-5" />
          </div>
          <span class="text-[10px] font-mono font-bold uppercase px-2 py-0.5 rounded bg-white/5 text-neutral-400">
            System Spec
          </span>
        </div>
        <h3 class="font-bold text-sm text-white">Execution Parameters</h3>
        
        <div class="mt-3 space-y-2 text-xs font-mono text-neutral-400">
          <div class="flex justify-between items-center py-0.5 border-b border-white/5">
            <span class="text-neutral-500">Allocated RAM</span>
            <span class="text-neutral-200 font-bold">{launcher.activeInstance?.max_memory_mb || launcher.settings.default_max_memory_mb} MB</span>
          </div>
          <div class="flex justify-between items-center py-0.5 border-b border-white/5">
            <span class="text-neutral-500">GC Preset</span>
            <span class="text-red-400 font-bold uppercase">{launcher.settings.performance_preset}</span>
          </div>
          <div class="flex justify-between items-center py-0.5">
            <span class="text-neutral-500">Java Version</span>
            <span class="text-neutral-200 font-bold">Temurin {launcher.detectedJavas.find(j => j.is_recommended)?.major_version || "Auto"}</span>
          </div>
        </div>
      </div>

      <button
        onclick={() => launcher.activeTab = "settings"}
        class="mt-4 w-full py-2.5 rounded-xl bg-white/5 hover:bg-white/10 border border-white/8 text-neutral-300 hover:text-white text-xs font-bold font-mono uppercase tracking-wider flex items-center justify-center gap-1.5 transition-all cursor-pointer"
      >
        <span>Configure Engine</span>
        <ArrowUpRight class="w-3.5 h-3.5" />
      </button>
    </div>
  </div>
</div>
