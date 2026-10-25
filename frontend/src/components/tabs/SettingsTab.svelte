<script lang="ts">
  import { launcher } from "../../state/launcher.svelte";
  import {
    Settings,
    Coffee,
    Cpu,
    HardDrive,
    Save,
    Download,
    Check,
    FolderOpen,
    FolderTree,
    Radio,
    Sparkles,
    RefreshCw,
    RotateCcw,
    FileText,
  } from "lucide-svelte";

  let savedSuccess = $state(false);

  async function handleSave() {
    await launcher.saveSettings();
    savedSuccess = true;
    setTimeout(() => {
      savedSuccess = false;
    }, 2500);
  }
</script>

<div class="h-full overflow-y-auto p-4 sm:p-6 lg:p-8 space-y-5 sm:space-y-6">
  <!-- Top Bar -->
  <div class="flex items-center justify-between pb-4 border-b border-white/8">
    <div>
      <h2
        class="text-2xl font-black text-white uppercase tracking-tight flex items-center gap-3"
      >
        <Settings class="w-6 h-6 text-red-500" />
        <span>Settings</span>
      </h2>
      <p class="text-xs text-neutral-300 mt-1 font-medium">
        Configure game storage directory, JVM performance, multi-version Java runtimes, and GPU acceleration.
      </p>
    </div>

    <button
      onclick={handleSave}
      class="px-6 py-2.5 rounded-xl bg-gradient-to-r from-red-600 to-rose-600 hover:from-red-500 hover:to-rose-500 text-white font-bold text-xs uppercase tracking-wider flex items-center gap-2 shadow-lg shadow-red-500/20 transition-all cursor-pointer hover:scale-105 active:scale-95"
    >
      {#if savedSuccess}
        <Check class="w-4 h-4" />
        <span>SAVED!</span>
      {:else}
        <Save class="w-4 h-4" />
        <span>SAVE SETTINGS</span>
      {/if}
    </button>
  </div>

  <!-- Game & Storage Directory Section (Prevent C: drive pileup) -->
  <div class="p-6 rounded-2xl bg-[#0e1017]/90 border border-white/8 space-y-5 shadow-xl">
    <div class="flex items-center justify-between pb-3 border-b border-white/8">
      <div class="flex items-center gap-3">
        <div class="w-10 h-10 rounded-xl bg-red-500/10 border border-red-500/20 flex items-center justify-center text-red-400 shrink-0">
          <HardDrive class="w-5 h-5" />
        </div>
        <div>
          <h3 class="text-sm font-bold uppercase tracking-wider text-white flex items-center gap-2">
            <span>Game & Storage Directories</span>
            <span class="px-2 py-0.5 rounded-full bg-red-500/20 text-red-300 text-[10px] font-black uppercase">
              Multi-Drive Support
            </span>
          </h3>
          <p class="text-xs text-neutral-300 mt-0.5">
            Store Minecraft assets, runtimes, and instances on ANY custom drive (D:, E:, etc.) to avoid filling your C: drive.
          </p>
        </div>
      </div>
    </div>

    <div class="grid grid-cols-1 gap-4">
      <!-- 1. Base Game Directory -->
      <div class="p-4 rounded-xl bg-white/2 border border-white/6 space-y-2">
        <div class="flex items-center justify-between">
          <div>
            <div class="text-xs text-neutral-200 font-bold block uppercase tracking-wide">
              Base Game & Core Assets Directory
            </div>
            <p class="text-[11px] text-neutral-400">
              Shared Minecraft versions, libraries, runtime environments, and launcher cache.
            </p>
          </div>
          <button
            onclick={() => launcher.resetGameDirectory()}
            class="text-[11px] text-neutral-400 hover:text-white transition-colors cursor-pointer"
            title="Reset base data directory to %APPDATA%\DigitalVigital"
          >
            Reset Default
          </button>
        </div>
        <div class="flex items-center gap-2 pt-1">
          <input
            type="text"
            readonly
            value={launcher.settings.data_dir || "Loading directory..."}
            class="flex-1 px-4 py-2.5 rounded-xl bg-white/4 border border-white/10 text-xs font-mono text-neutral-200 outline-none select-all"
          />
          <button
            onclick={() => launcher.pickGameDirectory()}
            class="px-4 py-2.5 rounded-xl bg-red-600 hover:bg-red-500 text-white font-bold text-xs uppercase tracking-wider flex items-center gap-2 transition-all cursor-pointer shadow-md shadow-red-600/30 shrink-0 active:scale-95"
            title="Browse and select base folder or drive"
          >
            <FolderOpen class="w-4 h-4" />
            <span>Browse / Change</span>
          </button>
          <button
            onclick={() => launcher.openFolder(launcher.settings.data_dir)}
            class="p-2.5 rounded-xl bg-white/5 hover:bg-white/10 text-neutral-300 hover:text-white transition-all cursor-pointer shrink-0 border border-white/10"
            title="Open folder in File Explorer"
          >
            <FolderOpen class="w-4 h-4" />
          </button>
        </div>
      </div>

      <!-- 2. Minecraft Instances Directory -->
      <div class="p-4 rounded-xl bg-white/2 border border-white/6 space-y-2">
        <div class="flex items-center justify-between">
          <div>
            <div class="text-xs text-neutral-200 font-bold block uppercase tracking-wide flex items-center gap-2">
              <FolderTree class="w-3.5 h-3.5 text-cyan-400" />
              <span>Minecraft Instances Directory</span>
              <span class="px-1.5 py-0.5 rounded bg-cyan-500/15 text-cyan-300 text-[10px] font-bold">Independent Path</span>
            </div>
            <p class="text-[11px] text-neutral-400">
              Where your Minecraft game profiles, worlds, mod configurations, and resource packs live.
            </p>
          </div>
          <button
            onclick={() => launcher.resetInstancesDirectory()}
            class="text-[11px] text-neutral-400 hover:text-white transition-colors cursor-pointer"
            title="Reset instances directory to default"
          >
            Reset Default
          </button>
        </div>
        <div class="flex items-center gap-2 pt-1">
          <input
            type="text"
            readonly
            value={launcher.settings.instances_dir || "Loading directory..."}
            class="flex-1 px-4 py-2.5 rounded-xl bg-white/4 border border-white/10 text-xs font-mono text-cyan-200 outline-none select-all"
          />
          <button
            onclick={() => launcher.pickInstancesDirectory()}
            class="px-4 py-2.5 rounded-xl bg-cyan-600 hover:bg-cyan-500 text-white font-bold text-xs uppercase tracking-wider flex items-center gap-2 transition-all cursor-pointer shadow-md shadow-cyan-600/30 shrink-0 active:scale-95"
            title="Browse and select instances folder or drive"
          >
            <FolderTree class="w-4 h-4" />
            <span>Browse / Change</span>
          </button>
          <button
            onclick={() => launcher.openFolder(launcher.settings.instances_dir)}
            class="p-2.5 rounded-xl bg-white/5 hover:bg-white/10 text-neutral-300 hover:text-white transition-all cursor-pointer shrink-0 border border-white/10"
            title="Open instances folder in File Explorer"
          >
            <FolderOpen class="w-4 h-4" />
          </button>
        </div>
      </div>
    </div>
  </div>

  <!-- Multi-Java Runtime Suite & Selection (Choose which Java is used) -->
  <div class="p-6 rounded-2xl bg-[#0e1017]/90 border border-white/8 space-y-6 shadow-xl">
    <!-- Header -->
    <div class="flex flex-col md:flex-row md:items-center justify-between gap-3 pb-3 border-b border-white/8">
      <div class="flex items-center gap-3">
        <div class="w-10 h-10 rounded-xl bg-amber-500/10 border border-amber-500/20 flex items-center justify-center text-amber-400 shrink-0">
          <Coffee class="w-5 h-5" />
        </div>
        <div>
          <h3 class="text-sm font-bold uppercase tracking-wider text-white flex items-center gap-2">
            <span>Java Runtimes & Execution Engine</span>
            <span class="px-2 py-0.5 rounded-full bg-amber-500/20 text-amber-300 text-[10px] font-black uppercase">
              Configurable
            </span>
          </h3>
          <p class="text-xs text-neutral-300 mt-0.5">
            Select which Java runtime executes your game or let the launcher dynamically choose the best JVM per Minecraft version.
          </p>
        </div>
      </div>

      <div class="flex items-center gap-2">
        <button
          type="button"
          onclick={() => launcher.scanForJava()}
          class="px-3 py-2 rounded-xl bg-white/5 hover:bg-white/10 border border-white/10 text-neutral-300 hover:text-white text-xs font-bold transition-all cursor-pointer flex items-center gap-1.5"
          title="Scan system and runtimes folder for Java installations"
        >
          <RefreshCw class="w-3.5 h-3.5" />
          <span>Scan System</span>
        </button>

        <button
          type="button"
          onclick={() => launcher.pickCustomJava()}
          class="px-3 py-2 rounded-xl bg-amber-600 hover:bg-amber-500 text-white text-xs font-bold transition-all cursor-pointer flex items-center gap-1.5 shadow-md shadow-amber-600/30"
          title="Browse file explorer for custom javaw.exe executable"
        >
          <FolderOpen class="w-3.5 h-3.5" />
          <span>Browse Java...</span>
        </button>
      </div>
    </div>

    <!-- Active Java Selection Highlight Banner -->
    <div class="p-4 rounded-xl bg-white/2 border border-white/8 flex flex-col md:flex-row md:items-center justify-between gap-4">
      <div class="flex items-center gap-3">
        <div class="w-9 h-9 rounded-lg bg-emerald-500/10 border border-emerald-500/20 flex items-center justify-center text-emerald-400 shrink-0">
          <Check class="w-4 h-4" />
        </div>
        <div>
          <div class="text-[10px] text-neutral-400 uppercase font-black tracking-wider flex items-center gap-2">
            <span>Active Selection:</span>
            {#if launcher.settings.custom_java_path}
              <span class="px-1.5 py-0.5 rounded bg-amber-500/20 text-amber-300 font-bold">Custom Override</span>
            {:else}
              <span class="px-1.5 py-0.5 rounded bg-emerald-500/20 text-emerald-300 font-bold">Auto-Detect (Smart)</span>
            {/if}
          </div>
          <div class="text-sm font-bold text-white mt-0.5 flex items-center gap-2">
            {#if launcher.settings.custom_java_path}
              <span>{launcher.detectedJavas.find((j) => j.path === launcher.settings.custom_java_path)?.display_name || "Custom Java Path"}</span>
            {:else}
              <span>Auto-Detect (Smart Version Resolution per Minecraft Edition)</span>
            {/if}
          </div>
          {#if launcher.settings.custom_java_path}
            <div class="text-[11px] font-mono text-neutral-400 truncate max-w-lg mt-0.5">
              {launcher.settings.custom_java_path}
            </div>
          {:else}
            <div class="text-[11px] text-neutral-400 mt-0.5">
              MC ≤1.16 &rarr; Java 8 (Legacy) • MC 1.17–1.20.4 &rarr; Java 17 • MC 1.20.5+ &rarr; Java 21
            </div>
          {/if}
        </div>
      </div>

      {#if launcher.settings.custom_java_path}
        <button
          type="button"
          onclick={() => launcher.selectJava(null)}
          class="px-3 py-1.5 rounded-lg bg-white/6 hover:bg-white/12 border border-white/10 text-neutral-300 hover:text-white text-xs font-semibold flex items-center gap-1.5 transition-all cursor-pointer shrink-0"
        >
          <RotateCcw class="w-3.5 h-3.5" />
          <span>Reset to Auto-Detect</span>
        </button>
      {/if}
    </div>

    <!-- Available Java Selection List -->
    <div class="space-y-2.5">
      <div class="text-xs text-neutral-300 font-bold uppercase tracking-wider">
        Choose Which Java to Use:
      </div>

      <div class="grid grid-cols-1 gap-2.5">
        <!-- Option 1: Auto-Detect (Default & Recommended) -->
        <button
          type="button"
          onclick={() => launcher.selectJava(null)}
          class="w-full p-4 rounded-xl text-left transition-all cursor-pointer flex items-center justify-between border {!launcher.settings.custom_java_path
            ? 'bg-emerald-500/10 border-emerald-500/40 shadow-lg shadow-emerald-500/10'
            : 'bg-white/2 hover:bg-white/5 border-white/6'}"
        >
          <div class="flex items-center gap-3">
            <div class="w-5 h-5 rounded-full border flex items-center justify-center shrink-0 {!launcher.settings.custom_java_path
              ? 'border-emerald-400 bg-emerald-500'
              : 'border-white/20 bg-black/40'}">
              {#if !launcher.settings.custom_java_path}
                <Check class="w-3 h-3 text-black font-black" />
              {/if}
            </div>
            <div>
              <div class="text-xs font-bold text-white flex items-center gap-2">
                <span>Auto-Detect & Route (Recommended)</span>
                <span class="px-2 py-0.5 rounded bg-emerald-500/20 text-emerald-300 text-[10px] font-black uppercase">
                  Default
                </span>
              </div>
              <p class="text-[11px] text-neutral-400 mt-0.5">
                Automatically matches the required Java runtime based on the Minecraft instance being launched.
              </p>
            </div>
          </div>

          <span class="text-xs font-bold {!launcher.settings.custom_java_path ? 'text-emerald-400' : 'text-neutral-500'}">
            {!launcher.settings.custom_java_path ? "In Use" : "Select"}
          </span>
        </button>

        <!-- Option 2+: Detected Java Installations -->
        {#each launcher.detectedJavas as j}
          {@const isSelected = launcher.settings.custom_java_path === j.path}
          <button
            type="button"
            onclick={() => launcher.selectJava(j.path)}
            class="w-full p-4 rounded-xl text-left transition-all cursor-pointer flex items-center justify-between border {isSelected
              ? 'bg-amber-500/10 border-amber-500/40 shadow-lg shadow-amber-500/10'
              : 'bg-white/2 hover:bg-white/5 border-white/6'}"
          >
            <div class="flex items-center gap-3">
              <div class="w-5 h-5 rounded-full border flex items-center justify-center shrink-0 {isSelected
                ? 'border-amber-400 bg-amber-500'
                : 'border-white/20 bg-black/40'}">
                {#if isSelected}
                  <Check class="w-3 h-3 text-black font-black" />
                {/if}
              </div>
              <div>
                <div class="text-xs font-bold text-white flex items-center gap-2">
                  <span>{j.display_name}</span>
                  <span class="px-2 py-0.5 rounded bg-amber-500/20 text-amber-300 text-[10px] font-black uppercase font-mono">
                    Java {j.major_version}
                  </span>
                  {#if j.is_recommended}
                    <span class="px-2 py-0.5 rounded bg-cyan-500/15 text-cyan-300 text-[10px] font-bold">
                      Recommended
                    </span>
                  {/if}
                </div>
                <div class="text-[11px] font-mono text-neutral-400 truncate max-w-lg mt-0.5">
                  {j.path}
                </div>
              </div>
            </div>

            <span class="text-xs font-bold {isSelected ? 'text-amber-400' : 'text-neutral-500'}">
              {isSelected ? "In Use" : "Select"}
            </span>
          </button>
        {/each}

        <!-- Custom Unlisted Path (if user picked custom not in detected list) -->
        {#if launcher.settings.custom_java_path && !launcher.detectedJavas.some((j) => j.path === launcher.settings.custom_java_path)}
          <div class="w-full p-4 rounded-xl text-left flex items-center justify-between border bg-amber-500/10 border-amber-500/40">
            <div class="flex items-center gap-3">
              <div class="w-5 h-5 rounded-full border border-amber-400 bg-amber-500 flex items-center justify-center shrink-0">
                <Check class="w-3 h-3 text-black font-black" />
              </div>
              <div>
                <div class="text-xs font-bold text-white flex items-center gap-2">
                  <span>Custom Executable</span>
                  <span class="px-2 py-0.5 rounded bg-amber-500/20 text-amber-300 text-[10px] font-black uppercase">
                    Custom Path
                  </span>
                </div>
                <div class="text-[11px] font-mono text-neutral-300 truncate max-w-lg mt-0.5">
                  {launcher.settings.custom_java_path}
                </div>
              </div>
            </div>
            <span class="text-xs font-bold text-amber-400">In Use</span>
          </div>
        {/if}
      </div>
    </div>

    <!-- 1-Click Java Download Cards (Temurin OpenJDK 8, 17, 21) -->
    <div class="pt-2 border-t border-white/8 space-y-3">
      <div class="flex items-center justify-between">
        <span class="text-xs text-neutral-300 font-bold uppercase tracking-wider">
          Install Missing Java Runtimes (1-Click)
        </span>
        <span class="text-[11px] text-neutral-400">
          Isolated launcher runtimes, no admin rights required
        </span>
      </div>

      <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
        <!-- Java 8 Card -->
        <div class="p-4 rounded-xl bg-white/2 border border-white/6 flex flex-col justify-between">
          <div>
            <div class="flex items-center justify-between mb-1.5">
              <span class="text-xs font-bold text-white">Java 8 (LTS)</span>
              {#if launcher.detectedJavas.some((j) => j.major_version === 8)}
                <span class="text-[10px] px-2 py-0.5 rounded bg-emerald-500/15 text-emerald-400 border border-emerald-500/25 font-bold uppercase">
                  Installed
                </span>
              {/if}
            </div>
            <p class="text-[11px] text-neutral-400">For Minecraft 1.8.9, 1.12.2, 1.16.5</p>
          </div>
          <button
            onclick={() => launcher.installJava(8)}
            disabled={launcher.isInstallingJava}
            class="mt-3 w-full py-2 rounded-lg bg-white/6 hover:bg-white/12 border border-white/10 text-xs font-bold text-white flex items-center justify-center gap-2 transition-all cursor-pointer disabled:opacity-50"
          >
            <Download class="w-3.5 h-3.5 text-amber-400" />
            <span>Install Java 8</span>
          </button>
        </div>

        <!-- Java 17 Card -->
        <div class="p-4 rounded-xl bg-white/2 border border-white/6 flex flex-col justify-between">
          <div>
            <div class="flex items-center justify-between mb-1.5">
              <span class="text-xs font-bold text-white">Java 17 (LTS)</span>
              {#if launcher.detectedJavas.some((j) => j.major_version === 17)}
                <span class="text-[10px] px-2 py-0.5 rounded bg-emerald-500/15 text-emerald-400 border border-emerald-500/25 font-bold uppercase">
                  Installed
                </span>
              {/if}
            </div>
            <p class="text-[11px] text-neutral-400">For Minecraft 1.17 - 1.20.4</p>
          </div>
          <button
            onclick={() => launcher.installJava(17)}
            disabled={launcher.isInstallingJava}
            class="mt-3 w-full py-2 rounded-lg bg-white/6 hover:bg-white/12 border border-white/10 text-xs font-bold text-white flex items-center justify-center gap-2 transition-all cursor-pointer disabled:opacity-50"
          >
            <Download class="w-3.5 h-3.5 text-amber-400" />
            <span>Install Java 17</span>
          </button>
        </div>

        <!-- Java 21 Card -->
        <div class="p-4 rounded-xl bg-white/2 border border-white/6 flex flex-col justify-between">
          <div>
            <div class="flex items-center justify-between mb-1.5">
              <span class="text-xs font-bold text-white">Java 21 (LTS)</span>
              {#if launcher.detectedJavas.some((j) => j.major_version === 21)}
                <span class="text-[10px] px-2 py-0.5 rounded bg-emerald-500/15 text-emerald-400 border border-emerald-500/25 font-bold uppercase">
                  Installed
                </span>
              {/if}
            </div>
            <p class="text-[11px] text-neutral-400">For Minecraft 1.20.5 - 1.21.4+</p>
          </div>
          <button
            onclick={() => launcher.installJava(21)}
            disabled={launcher.isInstallingJava}
            class="mt-3 w-full py-2 rounded-lg bg-white/6 hover:bg-white/12 border border-white/10 text-xs font-bold text-white flex items-center justify-center gap-2 transition-all cursor-pointer disabled:opacity-50"
          >
            <Download class="w-3.5 h-3.5 text-amber-400" />
            <span>Install Java 21</span>
          </button>
        </div>
      </div>
    </div>

    <!-- Java Install Progress Banner -->
    {#if launcher.isInstallingJava}
      <div class="p-4 rounded-xl bg-amber-500/10 border border-amber-500/20 space-y-2">
        <div class="flex items-center justify-between text-xs font-medium text-amber-300">
          <span>{launcher.javaInstallStatus}</span>
          <span>{launcher.javaInstallPercent}%</span>
        </div>
        <div class="w-full h-2 rounded-full bg-white/10 overflow-hidden">
          <div
            class="h-full bg-amber-400 rounded-full transition-all duration-200"
            style="width: {launcher.javaInstallPercent}%"
          ></div>
        </div>
      </div>
    {/if}
  </div>

  <!-- Memory & Performance Presets -->
  <div class="p-6 rounded-2xl bg-[#0e1017]/90 border border-white/8 space-y-5">
    <h3
      class="text-sm font-bold uppercase tracking-wider text-white flex items-center gap-2"
    >
      <HardDrive class="w-4 h-4 text-purple-400" />
      <span>Memory & Garbage Collection Optimization</span>
    </h3>

    <!-- RAM Sliders -->
    <div class="space-y-4">
      <div>
        <div class="flex justify-between items-center text-xs mb-2">
          <span class="text-neutral-300 font-medium"
            >Maximum RAM Allocation (Xmx)</span
          >
          <span class="text-red-400 font-bold font-mono text-sm"
            >{launcher.settings.default_max_memory_mb || 4096} MB ({(
              (launcher.settings.default_max_memory_mb || 4096) / 1024
            ).toFixed(1)} GB)</span
          >
        </div>
        <input
          type="range"
          min="1024"
          max="32768"
          step="512"
          value={launcher.settings.default_max_memory_mb || 4096}
          oninput={(e) => {
            launcher.settings.default_max_memory_mb = parseInt((e.target as HTMLInputElement).value, 10);
          }}
          onchange={async (e) => {
            launcher.settings.default_max_memory_mb = parseInt((e.target as HTMLInputElement).value, 10);
            await launcher.saveSettings();
          }}
          class="w-full accent-red-500 cursor-pointer h-2 bg-white/10 rounded-lg"
        />
        <div
          class="flex justify-between text-[11px] text-neutral-400 mt-1.5 font-medium"
        >
          <span>1 GB</span>
          <span>4 GB (Default)</span>
          <span>8 GB (Recommended)</span>
          <span>16 GB</span>
          <span>32 GB</span>
        </div>

        <!-- 1-Click Fast Presets -->
        <div class="grid grid-cols-3 sm:grid-cols-6 gap-2 mt-3">
          {#each [2048, 4096, 6144, 8192, 12288, 16384] as ramVal}
            <button
              type="button"
              onclick={async () => {
                launcher.settings.default_max_memory_mb = ramVal;
                await launcher.saveSettings();
              }}
              class="py-1.5 px-2 rounded-lg text-xs font-mono font-bold transition-all cursor-pointer border {launcher.settings.default_max_memory_mb === ramVal
                ? 'bg-red-500/20 border-red-500/50 text-white shadow-md shadow-red-500/20'
                : 'bg-white/4 border-white/8 text-neutral-400 hover:text-white hover:bg-white/8'}"
            >
              {ramVal / 1024} GB
            </button>
          {/each}
        </div>
      </div>
    </div>

    <!-- GC Preset Selection -->
    <div class="pt-2">
      <div class="text-xs text-neutral-300 font-medium mb-2.5">
        JVM Performance & Garbage Collector Preset
      </div>
      <div class="grid grid-cols-1 md:grid-cols-3 gap-3">
        <button
          onclick={() => (launcher.settings.performance_preset = "aikar")}
          class="p-4 rounded-xl border text-left transition-all cursor-pointer {launcher
            .settings.performance_preset === 'aikar'
            ? 'bg-red-500/15 border-red-500/40 text-white'
            : 'bg-white/3 border-white/8 text-neutral-300 hover:text-white'}"
        >
          <div class="text-xs font-bold uppercase tracking-wide">
            Aikar Flags (Recommended)
          </div>
          <div class="text-xs text-neutral-400 mt-1">
            High TPS, eliminates micro-stuttering on G1GC
          </div>
        </button>

        <button
          onclick={() => (launcher.settings.performance_preset = "zgc")}
          class="p-4 rounded-xl border text-left transition-all cursor-pointer {launcher
            .settings.performance_preset === 'zgc'
            ? 'bg-red-500/15 border-red-500/40 text-white'
            : 'bg-white/3 border-white/8 text-neutral-300 hover:text-white'}"
        >
          <div class="text-xs font-bold uppercase tracking-wide">
            Generational ZGC
          </div>
          <div class="text-xs text-neutral-400 mt-1">
            Sub-millisecond pause times for 6GB+ RAM
          </div>
        </button>

        <button
          onclick={() => (launcher.settings.performance_preset = "vanilla")}
          class="p-4 rounded-xl border text-left transition-all cursor-pointer {launcher
            .settings.performance_preset === 'vanilla'
            ? 'bg-red-500/15 border-red-500/40 text-white'
            : 'bg-white/3 border-white/8 text-neutral-300 hover:text-white'}"
        >
          <div class="text-xs font-bold uppercase tracking-wide">
            Vanilla Standard
          </div>
          <div class="text-xs text-neutral-400 mt-1">
            Default Java standard arguments
          </div>
        </button>
      </div>
    </div>

    <!-- Custom JVM Args -->
    <div>
      <div class="text-xs text-neutral-300 font-medium mb-2">
        Additional JVM Arguments
      </div>
      <input
        type="text"
        bind:value={launcher.settings.custom_jvm_args}
        placeholder="-XX:+AlwaysPreTouch -Dsun.rmi.dgc.server.gcInterval=2147483646"
        class="w-full px-4 py-2.5 rounded-xl bg-white/5 border border-white/10 text-xs font-mono text-white placeholder-neutral-500 outline-none focus:border-red-500/50"
      />
    </div>
  </div>

  <!-- Hardware GPU Acceleration -->
  <div class="p-6 rounded-2xl bg-[#0e1017]/90 border border-white/8 space-y-4">
    <h3
      class="text-sm font-bold uppercase tracking-wider text-white flex items-center gap-2"
    >
      <Cpu class="w-4 h-4 text-sky-400" />
      <span>Hardware GPU Preference (DirectX / OpenGL Acceleration)</span>
    </h3>

    <div class="grid grid-cols-1 md:grid-cols-2 gap-3">
      <button
        onclick={() => (launcher.settings.selected_gpu = "auto")}
        class="p-4 rounded-xl border text-left transition-all cursor-pointer {launcher
          .settings.selected_gpu === 'auto'
          ? 'bg-red-500/15 border-red-500/40 text-white'
          : 'bg-white/3 border-white/8 text-neutral-300 hover:text-white'}"
      >
        <div class="text-xs font-bold">Auto Priority (Discrete GPU first)</div>
        <div class="text-xs text-neutral-400 mt-1">
          Directs Windows to enforce High Performance GPU for javaw.exe
        </div>
      </button>

      {#each launcher.detectedGpus as gpu}
        <button
          onclick={() => (launcher.settings.selected_gpu = gpu.id)}
          class="p-4 rounded-xl border text-left transition-all cursor-pointer {launcher
            .settings.selected_gpu === gpu.id
            ? 'bg-red-500/15 border-red-500/40 text-white'
            : 'bg-white/3 border-white/8 text-neutral-300 hover:text-white'}"
        >
          <div class="text-xs font-bold truncate">{gpu.name}</div>
          <div class="text-xs text-neutral-400 mt-1">
            {gpu.is_discrete
              ? "Dedicated High Performance GPU"
              : "Integrated GPU"}
          </div>
        </button>
      {/each}
    </div>
  </div>

  <!-- Launcher Behavior -->
  <div class="p-6 rounded-2xl bg-[#0e1017]/90 border border-white/8 space-y-4">
    <h3
      class="text-sm font-bold uppercase tracking-wider text-white flex items-center gap-2"
    >
      <Radio class="w-4 h-4 text-rose-400" />
      <span>Launcher Window Behavior & Discord RPC</span>
    </h3>

    <div class="grid grid-cols-1 md:grid-cols-3 gap-3">
      <button
        onclick={() => (launcher.settings.on_launch_action = "hide")}
        class="p-4 rounded-xl border text-left transition-all cursor-pointer {launcher
          .settings.on_launch_action === 'hide'
          ? 'bg-red-500/15 border-red-500/40 text-white'
          : 'bg-white/3 border-white/8 text-neutral-300 hover:text-white'}"
      >
        <div class="text-xs font-bold">Hide on Launch</div>
        <div class="text-xs text-neutral-400 mt-1">
          Restores automatically when Minecraft exits
        </div>
      </button>

      <button
        onclick={() => (launcher.settings.on_launch_action = "keep")}
        class="p-4 rounded-xl border text-left transition-all cursor-pointer {launcher
          .settings.on_launch_action === 'keep'
          ? 'bg-red-500/15 border-red-500/40 text-white'
          : 'bg-white/3 border-white/8 text-neutral-300 hover:text-white'}"
      >
        <div class="text-xs font-bold">Keep Open</div>
        <div class="text-xs text-neutral-400 mt-1">
          Leaves the launcher open for console monitoring
        </div>
      </button>

      <button
        onclick={() => (launcher.settings.on_launch_action = "close")}
        class="p-4 rounded-xl border text-left transition-all cursor-pointer {launcher
          .settings.on_launch_action === 'close'
          ? 'bg-red-500/15 border-red-500/40 text-white'
          : 'bg-white/3 border-white/8 text-neutral-300 hover:text-white'}"
      >
        <div class="text-xs font-bold">Close Launcher</div>
        <div class="text-xs text-neutral-400 mt-1">
          Terminates launcher process to free 100% RAM
        </div>
      </button>
    </div>

    <!-- Discord RPC toggle -->
    <div class="pt-2 flex items-center justify-between">
      <div>
        <div class="text-sm font-bold text-white">
          Discord Rich Presence (RPC)
        </div>
        <div class="text-xs text-neutral-300">
          Shows "Playing Minecraft via Digital Vigital" in your Discord status
        </div>
      </div>
      <input
        type="checkbox"
        bind:checked={launcher.settings.enable_discord_rpc}
        class="w-5 h-5 accent-red-500 rounded cursor-pointer"
      />
    </div>
  </div>

  <!-- Software & Launcher Updates Section -->
  <div
    class="p-6 rounded-2xl bg-[#0e1017] border border-white/8 space-y-4 shadow-xl"
  >
    <div
      class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 pb-4 border-b border-white/6"
    >
      <div class="flex items-center gap-3">
        <div
          class="w-10 h-10 rounded-xl bg-gradient-to-br from-emerald-500/15 to-teal-500/15 border border-emerald-500/30 flex items-center justify-center text-emerald-400 shrink-0"
        >
          <Sparkles class="w-5 h-5" />
        </div>
        <div>
          <h3
            class="text-sm font-bold uppercase tracking-wider text-white flex items-center gap-2"
          >
            <span>Software & Updates</span>
            {#if launcher.updateInfo?.has_update}
              <span
                class="px-2 py-0.5 rounded-full bg-emerald-500/20 text-emerald-300 text-[10px] font-black uppercase"
              >
                Update Available
              </span>
            {:else}
              <span
                class="px-2 py-0.5 rounded-full bg-white/5 text-neutral-400 text-[10px] font-bold uppercase"
              >
                Up to Date
              </span>
            {/if}
          </h3>
          <p class="text-xs text-neutral-400 mt-0.5">
            Digital Vigital Launcher v{launcher.launcherInfo.version} • Automated GitHub Release Tracking
          </p>
        </div>
      </div>

      <div class="flex items-center gap-2 shrink-0">
        <button
          onclick={() => launcher.checkForUpdates(true)}
          disabled={launcher.isCheckingUpdates}
          class="px-4 py-2 rounded-xl bg-white/5 hover:bg-white/10 border border-white/10 text-neutral-200 hover:text-white text-xs font-bold flex items-center gap-2 transition-all cursor-pointer disabled:opacity-50"
        >
          <RefreshCw
            class="w-3.5 h-3.5 {launcher.isCheckingUpdates
              ? 'animate-spin text-emerald-400'
              : ''}"
          />
          <span>{launcher.isCheckingUpdates ? "Checking..." : "Check for Updates"}</span>
        </button>

        {#if launcher.updateInfo?.has_update}
          <button
            onclick={() => launcher.openReleaseUrl()}
            class="px-4 py-2 rounded-xl bg-gradient-to-r from-emerald-600 to-teal-600 hover:from-emerald-500 hover:to-teal-500 text-white text-xs font-bold flex items-center gap-1.5 transition-all cursor-pointer shadow-lg shadow-emerald-600/30"
          >
            <Download class="w-3.5 h-3.5" />
            <span>Download {launcher.updateInfo.latest_version}</span>
          </button>
        {/if}
      </div>
    </div>

    {#if launcher.updateInfo?.has_update}
      <div
        class="p-4 rounded-xl bg-emerald-500/10 border border-emerald-500/25 flex flex-col sm:flex-row items-start sm:items-center justify-between gap-3"
      >
        <div>
          <div class="text-xs font-bold text-emerald-300">
            Version {launcher.updateInfo.latest_version} is ready to install!
          </div>
          <div class="text-[11px] text-neutral-300 mt-0.5">
            Click Download Update to get the latest executable installer or portable archive from GitHub.
          </div>
        </div>

        <button
          onclick={() => (launcher.showUpdateModal = true)}
          class="px-3 py-1.5 rounded-lg bg-emerald-500/20 hover:bg-emerald-500/30 border border-emerald-500/40 text-emerald-200 text-xs font-bold flex items-center gap-1.5 transition-all cursor-pointer shrink-0"
        >
          <FileText class="w-3.5 h-3.5" />
          <span>View Release Notes</span>
        </button>
      </div>
    {/if}
  </div>
</div>
