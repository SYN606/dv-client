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
    Sliders,
    FolderTree,
    ArrowUpRight,
    AlertTriangle,
    Send,
    Loader2,
    Check,
    Terminal,
    Download,
    X,
    FileText,
  } from "lucide-svelte";

  function getLoaderColor(loader: string = "") {
    const l = loader.toLowerCase();
    if (l.includes("fabric"))
      return "text-purple-300 border-purple-500/30 bg-purple-500/15";
    if (l.includes("neoforge"))
      return "text-orange-300 border-orange-500/30 bg-orange-500/15";
    if (l.includes("forge"))
      return "text-amber-300 border-amber-500/30 bg-amber-500/15";
    return "text-neutral-300 border-neutral-500/30 bg-neutral-500/15";
  }
</script>

<div class="h-full overflow-y-auto p-8 space-y-6">
  <!-- Software Update Notification Banner -->
  {#if launcher.updateInfo?.has_update && !launcher.dismissedUpdateBanner}
    <div
      class="p-4 rounded-2xl bg-gradient-to-r from-emerald-950/70 via-[#0d1f17]/60 to-[#0a1611]/80 border border-emerald-500/35 flex flex-col md:flex-row items-start md:items-center justify-between gap-4 shadow-xl"
    >
      <div class="flex items-center gap-3.5">
        <div
          class="w-10 h-10 rounded-xl bg-emerald-500/20 border border-emerald-500/30 flex items-center justify-center text-emerald-400 shrink-0 shadow-md shadow-emerald-500/10"
        >
          <Sparkles class="w-5 h-5" />
        </div>
        <div>
          <div class="text-sm font-bold text-white flex items-center gap-2">
            <span>Digital Vigital {launcher.updateInfo.latest_version} is Available!</span>
            <span
              class="px-2 py-0.5 rounded-full bg-emerald-500/25 text-emerald-300 text-[10px] font-black uppercase tracking-wider"
            >
              Update Ready
            </span>
          </div>
          <p class="text-xs text-neutral-300 mt-0.5">
            You are running v{launcher.updateInfo.current_version}. Upgrade now for the latest features, fixes, and game optimizations.
          </p>
        </div>
      </div>

      <div class="flex items-center gap-2 shrink-0 w-full md:w-auto">
        <button
          onclick={() => launcher.openReleaseUrl()}
          class="flex-1 md:flex-none px-4 py-2 rounded-xl bg-gradient-to-r from-emerald-600 to-teal-600 hover:from-emerald-500 hover:to-teal-500 text-white font-bold text-xs uppercase tracking-wider flex items-center justify-center gap-2 transition-all cursor-pointer shadow-lg shadow-emerald-600/30"
        >
          <Download class="w-3.5 h-3.5" />
          <span>Download Update</span>
        </button>

        <button
          onclick={() => (launcher.showUpdateModal = true)}
          class="px-3.5 py-2 rounded-xl bg-white/10 hover:bg-white/15 text-neutral-200 hover:text-white font-bold text-xs flex items-center gap-1.5 transition-all cursor-pointer border border-white/10"
        >
          <FileText class="w-3.5 h-3.5" />
          <span>Changelog</span>
        </button>

        <button
          onclick={() => (launcher.dismissedUpdateBanner = true)}
          class="p-2 rounded-xl bg-white/5 hover:bg-white/10 text-neutral-400 hover:text-white transition-all cursor-pointer"
          title="Dismiss notification for this session"
        >
          <X class="w-4 h-4" />
        </button>
      </div>
    </div>
  {/if}

  <!-- Main Launch Hero Card -->
  <div
    class="relative rounded-3xl overflow-hidden border border-white/10 bg-gradient-to-br from-[#12141c] via-[#0c0d14] to-[#18090d] shadow-2xl p-8 flex flex-col justify-between min-h-[300px]"
  >
    <!-- Ambient Red Specular Flare -->
    <div
      class="absolute -top-24 -right-24 w-96 h-96 bg-red-600/15 rounded-full blur-3xl pointer-events-none"
    ></div>
    <div
      class="absolute bottom-0 right-1/4 w-80 h-40 bg-purple-600/10 rounded-full blur-3xl pointer-events-none"
    ></div>

    <!-- Top Info Row in Hero -->
    <div class="relative z-10 flex items-start justify-between">
      <div>
        <div
          class="inline-flex items-center gap-2 px-3.5 py-1.5 rounded-full bg-red-500/10 border border-red-500/20 text-red-400 text-xs font-bold tracking-wide uppercase mb-3"
        >
          <Zap class="w-3.5 h-3.5 fill-red-400" />
          <span>Ultra-Low Latency Engine</span>
        </div>
        <h2 class="text-3xl font-extrabold text-white tracking-tight">
          {launcher.activeInstance
            ? launcher.activeInstance.name
            : "Select or Create Profile"}
        </h2>
        <div
          class="flex items-center gap-3 mt-2.5 text-xs text-neutral-300 font-medium"
        >
          {#if launcher.activeInstance}
            <span
              class="px-2.5 py-0.5 rounded border {getLoaderColor(
                launcher.activeInstance.mod_loader,
              )} font-bold uppercase tracking-wide"
            >
              {launcher.activeInstance.mod_loader}
            </span>
            <span class="text-neutral-500">•</span>
            <span class="text-white font-semibold"
              >Minecraft {launcher.activeInstance.mc_version}</span
            >
            <span class="text-neutral-500">•</span>
            <span class="text-neutral-300"
              >{launcher.installedMods.length} mods installed</span
            >
          {:else}
            <span class="text-neutral-400"
              >Pick an active profile to begin playing</span
            >
          {/if}
        </div>
      </div>

      <!-- Quick Action Pill for Mods & Folder -->
      {#if launcher.activeInstance}
        <div class="flex items-center gap-2">
          <button
            onclick={() => launcher.openModsFolder()}
            title="Open instance folder"
            class="px-3.5 py-2 rounded-xl bg-white/5 hover:bg-white/10 border border-white/10 text-neutral-200 hover:text-white text-xs font-semibold flex items-center gap-2 transition-all cursor-pointer"
          >
            <FolderOpen class="w-4 h-4" />
            <span>Folder</span>
          </button>
          <button
            onclick={() => (launcher.activeTab = "mods")}
            title="Open Mod Studio"
            class="px-3.5 py-2 rounded-xl bg-white/5 hover:bg-white/10 border border-white/10 text-neutral-200 hover:text-white text-xs font-semibold flex items-center gap-2 transition-all cursor-pointer"
          >
            <Wrench class="w-4 h-4" />
            <span>Mods</span>
          </button>
        </div>
      {/if}
    </div>

    <!-- Crash / Error Alert Card -->
    {#if launcher.gameStatus === "crashed" || launcher.hasCrashDetected}
      <div
        class="relative z-10 p-4.5 rounded-2xl bg-rose-950/70 border border-rose-500/50 flex flex-col gap-3 shadow-2xl backdrop-blur-md"
      >
        <div class="flex flex-col sm:flex-row items-start sm:items-center justify-between gap-4">
          <div class="flex items-start gap-3">
            <div
              class="w-10 h-10 rounded-xl bg-rose-500/20 border border-rose-500/30 flex items-center justify-center text-rose-400 shrink-0 mt-0.5"
            >
              <AlertTriangle class="w-5 h-5" />
            </div>
            <div>
              <div class="text-sm font-bold text-white flex items-center gap-2 flex-wrap">
                <span>{launcher.crashDetails?.headline || "Minecraft Exited Unexpectedly or Crashed"}</span>
                <span
                  class="px-2 py-0.5 rounded bg-rose-500/20 border border-rose-500/40 text-rose-300 text-[10px] font-black uppercase"
                >
                  {launcher.crashDetails?.category || "Crash Alert"}
                </span>
              </div>
              <p class="text-xs text-neutral-200 mt-1 leading-relaxed">
                {launcher.crashDetails?.explanation || "The game process terminated with an error. Review the Live Console or report diagnostics."}
              </p>
              {#if launcher.crashDetails?.recommended_action}
                <div class="mt-2 text-xs text-amber-300 bg-amber-950/40 border border-amber-500/30 rounded-lg px-3 py-1.5 flex items-center gap-2">
                  <span class="font-bold uppercase tracking-wider text-[10px] text-amber-400">Fix:</span>
                  <span>{launcher.crashDetails.recommended_action}</span>
                </div>
              {/if}
            </div>
          </div>

          <div class="flex items-center gap-2 w-full sm:w-auto shrink-0 flex-wrap justify-end">
            {#if launcher.crashDetails?.category?.includes("Java")}
              <button
                onclick={() => (launcher.activeTab = "settings")}
                class="px-3.5 py-2 rounded-xl bg-amber-600 hover:bg-amber-500 text-neutral-900 font-bold text-xs flex items-center gap-1.5 transition-all cursor-pointer shadow-md"
              >
                <span>Java Settings</span>
              </button>
            {:else if launcher.crashDetails?.category?.includes("Mod")}
              <button
                onclick={() => (launcher.activeTab = "mods")}
                class="px-3.5 py-2 rounded-xl bg-indigo-600 hover:bg-indigo-500 text-white font-bold text-xs flex items-center gap-1.5 transition-all cursor-pointer shadow-md"
              >
                <span>Mod Studio</span>
              </button>
            {/if}

            <button
              onclick={() => launcher.reportCrash()}
              disabled={launcher.isReportingCrash}
              class="px-4 py-2 rounded-xl bg-rose-600 hover:bg-rose-500 disabled:opacity-50 text-white font-bold text-xs uppercase tracking-wider flex items-center justify-center gap-2 transition-all cursor-pointer shadow-lg shadow-rose-600/30"
            >
              {#if launcher.isReportingCrash}
                <Loader2 class="w-3.5 h-3.5 animate-spin" />
                <span>Reporting...</span>
              {:else if launcher.crashReportSuccess}
                <Check class="w-3.5 h-3.5 text-emerald-300" />
                <span>Reported!</span>
              {:else}
                <Send class="w-3.5 h-3.5" />
                <span>Report Crash</span>
              {/if}
            </button>

            <button
              onclick={() => (launcher.activeTab = "console")}
              class="px-3.5 py-2 rounded-xl bg-white/10 hover:bg-white/15 text-neutral-200 hover:text-white font-bold text-xs flex items-center gap-1.5 transition-all cursor-pointer border border-white/10"
            >
              <Terminal class="w-3.5 h-3.5" />
              <span>Console</span>
            </button>
          </div>
        </div>
      </div>
    {/if}

    <!-- Bottom Launch Area -->
    <div
      class="relative z-10 pt-6 flex flex-col md:flex-row items-center justify-between gap-6"
    >
      <!-- Status & Download Progress -->
      <div class="w-full md:w-auto flex-1 space-y-2">
        <div class="flex items-center gap-2.5 text-sm text-neutral-200">
          {#if launcher.gameStatus === "running"}
            <span class="w-2.5 h-2.5 rounded-full bg-emerald-400 animate-pulse"
            ></span>
          {:else if launcher.gameStatus === "launching" || launcher.gameStatus === "downloading"}
            <span class="w-2.5 h-2.5 rounded-full bg-amber-400 animate-spin"
            ></span>
          {:else if launcher.gameStatus === "crashed"}
            <span class="w-2.5 h-2.5 rounded-full bg-rose-500 animate-pulse"
            ></span>
          {:else}
            <span class="w-2.5 h-2.5 rounded-full bg-neutral-600"></span>
          {/if}
          <span class="font-medium">{launcher.launchStatusText}</span>
        </div>

        {#if launcher.gameStatus === "downloading"}
          <div
            class="w-full max-w-md h-2.5 rounded-full bg-white/8 border border-white/10 overflow-hidden relative"
          >
            <div
              class="h-full bg-gradient-to-r from-red-600 to-rose-400 rounded-full transition-all duration-200"
              style="width: {launcher.downloadProgress.percent}%"
            ></div>
          </div>
          <div
            class="flex justify-between max-w-md text-xs font-mono text-neutral-400"
          >
            <span
              >{launcher.downloadProgress.current} / {launcher.downloadProgress
                .total} items</span
            >
            <span>{launcher.downloadProgress.percent.toFixed(1)}%</span>
          </div>
        {/if}
      </div>

      <!-- Main Giant Play / Stop Button -->
      <button
        onclick={() => launcher.launchOrStop()}
        disabled={launcher.gameStatus === "launching"}
        class="w-full md:w-auto px-10 py-4 rounded-2xl font-black uppercase tracking-wider text-sm flex items-center justify-center gap-3 shadow-2xl transition-all cursor-pointer duration-200 active:scale-95 disabled:opacity-50 disabled:cursor-not-allowed {launcher.gameStatus ===
        'running'
          ? 'bg-rose-600 hover:bg-rose-500 text-white shadow-rose-600/30 border border-rose-400/30'
          : launcher.gameStatus === 'downloading' ||
              launcher.gameStatus === 'launching'
            ? 'bg-amber-600 text-white cursor-wait border border-amber-400/30'
            : 'bg-gradient-to-r from-red-600 to-rose-600 hover:from-red-500 hover:to-rose-500 text-white shadow-red-600/40 hover:shadow-red-500/50 hover:scale-[1.02] border border-red-400/30'}"
      >
        {#if launcher.gameStatus === "running"}
          <Square class="w-5 h-5 fill-white" />
          <span>STOP MINECRAFT</span>
        {:else if launcher.gameStatus === "launching"}
          <div
            class="w-5 h-5 border-2 border-white/30 border-t-white rounded-full animate-spin"
          ></div>
          <span>LAUNCHING...</span>
        {:else if launcher.gameStatus === "downloading"}
          <div
            class="w-5 h-5 border-2 border-white/30 border-t-white rounded-full animate-spin"
          ></div>
          <span>DOWNLOADING...</span>
        {:else}
          <Play class="w-5 h-5 fill-white" />
          <span>PLAY MINECRAFT</span>
        {/if}
      </button>
    </div>
  </div>

  <!-- Quick Feature Grid -->
  <div class="grid grid-cols-1 md:grid-cols-3 gap-6">
    <!-- 1-Click Essential FPS Pack -->
    <div
      class="p-6 rounded-2xl bg-[#0e1017]/90 border border-white/8 hover:border-white/16 transition-all flex flex-col justify-between group"
    >
      <div>
        <div class="flex items-center justify-between mb-3">
          <div
            class="w-10 h-10 rounded-xl bg-red-500/10 border border-red-500/20 flex items-center justify-center text-red-400 group-hover:scale-110 transition-transform"
          >
            <Sparkles class="w-5 h-5" />
          </div>
          <span
            class="text-xs font-bold uppercase tracking-wider px-2.5 py-0.5 rounded bg-red-500/15 text-red-400 border border-red-500/20"
          >
            Turbo Spec
          </span>
        </div>
        <h3 class="font-bold text-base text-white">Essential FPS Suite</h3>
        <p class="text-xs text-neutral-300 mt-1.5 leading-relaxed font-normal">
          One-click install Sodium, Lithium, and FerriteCore for 3x higher FPS
          and stutter-free chunk loading.
        </p>
      </div>

      <button
        onclick={() => launcher.installEssentialFpsPack()}
        disabled={!launcher.activeInstance || launcher.installingModId !== ""}
        class="mt-5 w-full py-2.5 rounded-xl bg-white/5 hover:bg-red-500/20 hover:border-red-500/30 border border-white/10 text-white text-xs font-bold uppercase tracking-wider flex items-center justify-center gap-2 transition-all cursor-pointer disabled:opacity-50"
      >
        {#if launcher.installingModId}
          <div
            class="w-3.5 h-3.5 border-2 border-white/30 border-t-white rounded-full animate-spin"
          ></div>
          <span>INSTALLING...</span>
        {:else}
          <Zap class="w-3.5 h-3.5 text-red-400" />
          <span>1-CLICK FPS BOOST</span>
        {/if}
      </button>
    </div>

    <!-- Storage & Engine Management Card -->
    <div
      class="p-6 rounded-2xl bg-[#0e1017]/90 border border-white/8 hover:border-white/16 transition-all flex flex-col justify-between group"
    >
      <div>
        <div class="flex items-center justify-between mb-3">
          <div
            class="w-10 h-10 rounded-xl bg-cyan-500/10 border border-cyan-500/20 flex items-center justify-center text-cyan-400 group-hover:scale-110 transition-transform"
          >
            <FolderTree class="w-5 h-5" />
          </div>
          <span
            class="text-xs font-bold uppercase tracking-wider px-2.5 py-0.5 rounded bg-cyan-500/15 text-cyan-400 border border-cyan-500/20"
          >
            Multi-Drive
          </span>
        </div>
        <h3 class="font-bold text-base text-white">Storage & Engine Paths</h3>
        <p class="text-xs text-neutral-300 mt-1.5 leading-relaxed font-normal">
          Customize base game files and instance directories across your SSDs and HDDs.
        </p>
      </div>

      <button
        onclick={() => (launcher.activeTab = "settings")}
        class="mt-5 w-full py-2.5 rounded-xl bg-white/5 hover:bg-cyan-500/20 hover:border-cyan-500/30 border border-white/10 text-white text-xs font-bold uppercase tracking-wider flex items-center justify-center gap-2 transition-all cursor-pointer"
      >
        <Sliders class="w-4 h-4 text-cyan-400" />
        <span>Configure Storage</span>
      </button>
    </div>

    <!-- Active Hardware & Tuning Summary -->
    <div
      class="p-6 rounded-2xl bg-[#0e1017]/90 border border-white/8 hover:border-white/16 transition-all flex flex-col justify-between"
    >
      <div>
        <div class="flex items-center justify-between mb-3">
          <div
            class="w-10 h-10 rounded-xl bg-purple-500/10 border border-purple-500/20 flex items-center justify-center text-purple-400"
          >
            <HardDrive class="w-5 h-5" />
          </div>
          <span
            class="text-xs font-bold uppercase tracking-wider px-2.5 py-0.5 rounded bg-white/8 text-neutral-300 border border-white/10"
          >
            Engine Spec
          </span>
        </div>
        <h3 class="font-bold text-base text-white">Execution Parameters</h3>

        <div class="mt-3.5 space-y-2 text-xs text-neutral-300">
          <div
            class="flex justify-between items-center py-1 border-b border-white/5"
          >
            <span class="text-neutral-400">Allocated RAM</span>
            <span class="text-white font-bold font-mono"
              >{launcher.activeInstance?.max_memory_mb ||
                launcher.settings.default_max_memory_mb} MB</span
            >
          </div>
          <div
            class="flex justify-between items-center py-1 border-b border-white/5"
          >
            <span class="text-neutral-400">GC Preset</span>
            <span class="text-red-400 font-bold uppercase"
              >{launcher.settings.performance_preset}</span
            >
          </div>
          <div class="flex justify-between items-center py-1">
            <span class="text-neutral-400">Java Runtime</span>
            <span class="text-white font-bold font-mono">
              {#if launcher.settings.custom_java_path}
                {launcher.detectedJavas.find((j) => j.path === launcher.settings.custom_java_path)?.display_name || "Custom Java"}
              {:else}
                Auto-Detect
              {/if}
            </span>
          </div>
        </div>
      </div>

      <button
        onclick={() => (launcher.activeTab = "settings")}
        class="mt-5 w-full py-2.5 rounded-xl bg-white/5 hover:bg-white/10 border border-white/10 text-white text-xs font-bold uppercase tracking-wider flex items-center justify-center gap-1.5 transition-all cursor-pointer"
      >
        <span>Configure Engine</span>
        <ArrowUpRight class="w-4 h-4" />
      </button>
    </div>
  </div>
</div>
