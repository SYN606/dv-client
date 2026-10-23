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
    Radio,
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

<div class="h-full overflow-y-auto p-8 space-y-6">
  <!-- Top Bar -->
  <div class="flex items-center justify-between pb-4 border-b border-white/8">
    <div>
      <h2
        class="text-2xl font-black text-white uppercase tracking-tight flex items-center gap-3"
      >
        <Settings class="w-6 h-6 text-red-500" />
        <span>Java Runtime & Engine Settings</span>
      </h2>
      <p class="text-xs text-neutral-300 mt-1 font-medium">
        Fine-tune JVM performance, multi-version Java runtimes (8, 17, 21), and
        GPU acceleration.
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

  <!-- Multi-Java Runtime Suite (8, 17, 21) -->
  <div class="p-6 rounded-2xl bg-[#0e1017]/90 border border-white/8 space-y-5">
    <div
      class="flex flex-col md:flex-row md:items-center justify-between gap-2"
    >
      <div>
        <h3
          class="text-sm font-bold uppercase tracking-wider text-white flex items-center gap-2"
        >
          <Coffee class="w-4 h-4 text-amber-400" />
          <span>Multi-Version Java Runtimes (Temurin OpenJDK)</span>
        </h3>
        <p class="text-xs text-neutral-300 mt-1">
          Minecraft 1.8 - 1.16 requires Java 8; 1.17 requires Java 17; 1.20.5+
          requires Java 21.
        </p>
      </div>

      <button
        onclick={() => launcher.pickCustomJava()}
        class="px-4 py-2 rounded-xl bg-white/5 hover:bg-white/10 border border-white/10 text-neutral-200 hover:text-white text-xs font-bold transition-all cursor-pointer flex items-center gap-2"
      >
        <FolderOpen class="w-4 h-4" />
        <span>Custom Java...</span>
      </button>
    </div>

    <!-- 1-Click Java Download Cards -->
    <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
      <!-- Java 8 Card -->
      <div
        class="p-5 rounded-xl bg-white/4 border border-white/8 flex flex-col justify-between"
      >
        <div>
          <div class="flex items-center justify-between mb-2">
            <span class="text-sm font-bold text-white">Java 8 (LTS)</span>
            {#if launcher.detectedJavas.some((j) => j.major_version === 8)}
              <span
                class="text-xs px-2 py-0.5 rounded bg-emerald-500/15 text-emerald-400 border border-emerald-500/25 font-bold uppercase"
              >
                Installed
              </span>
            {/if}
          </div>
          <p class="text-xs text-neutral-300">
            For Minecraft 1.8.9, 1.12.2, 1.16.5
          </p>
        </div>
        <button
          onclick={() => launcher.installJava(8)}
          disabled={launcher.isInstallingJava}
          class="mt-4 w-full py-2.5 rounded-lg bg-white/6 hover:bg-white/12 border border-white/10 text-xs font-bold text-white flex items-center justify-center gap-2 transition-all cursor-pointer disabled:opacity-50"
        >
          <Download class="w-4 h-4 text-amber-400" />
          <span>Install Java 8</span>
        </button>
      </div>

      <!-- Java 17 Card -->
      <div
        class="p-5 rounded-xl bg-white/4 border border-white/8 flex flex-col justify-between"
      >
        <div>
          <div class="flex items-center justify-between mb-2">
            <span class="text-sm font-bold text-white">Java 17 (LTS)</span>
            {#if launcher.detectedJavas.some((j) => j.major_version === 17)}
              <span
                class="text-xs px-2 py-0.5 rounded bg-emerald-500/15 text-emerald-400 border border-emerald-500/25 font-bold uppercase"
              >
                Installed
              </span>
            {/if}
          </div>
          <p class="text-xs text-neutral-300">For Minecraft 1.17 - 1.20.4</p>
        </div>
        <button
          onclick={() => launcher.installJava(17)}
          disabled={launcher.isInstallingJava}
          class="mt-4 w-full py-2.5 rounded-lg bg-white/6 hover:bg-white/12 border border-white/10 text-xs font-bold text-white flex items-center justify-center gap-2 transition-all cursor-pointer disabled:opacity-50"
        >
          <Download class="w-4 h-4 text-amber-400" />
          <span>Install Java 17</span>
        </button>
      </div>

      <!-- Java 21 Card -->
      <div
        class="p-5 rounded-xl bg-white/4 border border-white/8 flex flex-col justify-between"
      >
        <div>
          <div class="flex items-center justify-between mb-2">
            <span class="text-sm font-bold text-white">Java 21 (LTS)</span>
            {#if launcher.detectedJavas.some((j) => j.major_version === 21)}
              <span
                class="text-xs px-2 py-0.5 rounded bg-emerald-500/15 text-emerald-400 border border-emerald-500/25 font-bold uppercase"
              >
                Installed
              </span>
            {/if}
          </div>
          <p class="text-xs text-neutral-300">For Minecraft 1.20.5 - 1.21.4+</p>
        </div>
        <button
          onclick={() => launcher.installJava(21)}
          disabled={launcher.isInstallingJava}
          class="mt-4 w-full py-2.5 rounded-lg bg-white/6 hover:bg-white/12 border border-white/10 text-xs font-bold text-white flex items-center justify-center gap-2 transition-all cursor-pointer disabled:opacity-50"
        >
          <Download class="w-4 h-4 text-amber-400" />
          <span>Install Java 21</span>
        </button>
      </div>
    </div>

    <!-- Java Install Progress Banner -->
    {#if launcher.isInstallingJava}
      <div
        class="p-4 rounded-xl bg-amber-500/10 border border-amber-500/20 space-y-2"
      >
        <div
          class="flex items-center justify-between text-xs font-medium text-amber-300"
        >
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
          <span class="text-red-400 font-bold font-mono"
            >{launcher.settings.default_max_memory_mb} MB ({(
              launcher.settings.default_max_memory_mb / 1024
            ).toFixed(1)} GB)</span
          >
        </div>
        <input
          type="range"
          min="1024"
          max="16384"
          step="512"
          bind:value={launcher.settings.default_max_memory_mb}
          class="w-full accent-red-500 cursor-pointer"
        />
        <div
          class="flex justify-between text-xs text-neutral-400 mt-1.5 font-medium"
        >
          <span>1 GB</span>
          <span>4 GB (Default)</span>
          <span>8 GB (Recommended)</span>
          <span>16 GB</span>
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
</div>
