<script lang="ts">
  import { launcher } from "../../state/launcher.svelte";
  import {
    Heart,
    ExternalLink,
    Zap,
    HardDrive,
    ShieldCheck,
    Coffee,
    Sparkles,
    Server,
    Globe,
    CheckCircle2,
    Code2,
    Cpu,
  } from "lucide-svelte";

  function openLink(url: string) {
    launcher.openFolder(url);
  }
</script>

<div class="h-full overflow-y-auto p-8 space-y-8 select-none">
  <!-- Hero Section -->
  <div
    class="relative rounded-3xl overflow-hidden border border-white/10 bg-gradient-to-br from-[#12141c] via-[#0c0d14] to-[#1a080d] p-8 shadow-2xl flex flex-col md:flex-row items-center justify-between gap-8"
  >
    <div class="flex items-center gap-6">
      <div
        class="relative w-24 h-24 rounded-3xl overflow-hidden bg-black/50 border border-white/12 shadow-2xl p-2 shrink-0 group hover:scale-105 transition-transform"
      >
        <img
          src="/128x128.png"
          alt="Digital Vigital Logo"
          class="w-full h-full object-contain"
        />
        <div
          class="absolute inset-0 bg-gradient-to-t from-red-600/20 to-transparent pointer-events-none"
        ></div>
      </div>

      <div>
        <div class="flex items-center gap-2 mb-1.5">
          <span
            class="px-2.5 py-0.5 rounded-full bg-red-500/15 border border-red-500/30 text-red-400 font-mono text-xs font-bold uppercase tracking-wider"
          >
            v{launcher.launcherInfo.version} Release
          </span>
          <span
            class="px-2.5 py-0.5 rounded-full bg-emerald-500/15 border border-emerald-500/30 text-emerald-400 text-xs font-bold"
          >
            100% Free & Open Source
          </span>
        </div>

        <h2 class="text-3xl font-black text-white tracking-tight uppercase">
          Digital Vigital (DV)
        </h2>
        <p class="text-neutral-300 text-sm mt-1 max-w-xl font-medium leading-relaxed">
          The next-generation, high-performance desktop Minecraft launcher built from the ground up in Rust. Zero bloat, zero ads, maximum FPS.
        </p>
      </div>
    </div>

    <!-- Quick Links -->
    <div class="flex flex-wrap md:flex-col gap-2.5 shrink-0 w-full md:w-auto">
      <button
        onclick={() => openLink("https://digitalvigital.fun/minecraft")}
        class="flex-1 md:flex-none px-4 py-2.5 rounded-xl bg-gradient-to-r from-red-600 to-rose-600 hover:from-red-500 hover:to-rose-500 text-white font-bold text-xs uppercase tracking-wider flex items-center justify-center gap-2 shadow-lg shadow-red-600/30 transition-all cursor-pointer hover:scale-105"
      >
        <Globe class="w-4 h-4" />
        <span>digitalvigital.fun/minecraft</span>
      </button>

      <button
        onclick={() => openLink("https://syn606.wtf")}
        class="flex-1 md:flex-none px-4 py-2.5 rounded-xl bg-white/5 hover:bg-white/10 border border-white/10 text-neutral-200 hover:text-white font-bold text-xs flex items-center justify-center gap-2 transition-all cursor-pointer"
      >
        <Code2 class="w-4 h-4 text-purple-400" />
        <span>Developer: syn606.wtf</span>
        <ExternalLink class="w-3.5 h-3.5 text-neutral-400" />
      </button>
    </div>
  </div>

  <!-- Developer Showcase Card -->
  <div
    class="p-6 rounded-2xl bg-gradient-to-r from-[#14121f] via-[#0e1017] to-[#180e14] border border-white/10 flex flex-col sm:flex-row items-start sm:items-center justify-between gap-5 shadow-xl"
  >
    <div class="flex items-center gap-4">
      <div
        class="w-14 h-14 rounded-2xl bg-gradient-to-br from-purple-500 to-indigo-600 flex items-center justify-center text-white font-black text-xl shadow-lg shadow-purple-500/25 shrink-0"
      >
        606
      </div>
      <div>
        <div class="flex items-center gap-2">
          <span class="text-xs font-bold uppercase tracking-wider text-purple-400">
            Lead Architect & Developer
          </span>
          <span class="w-1.5 h-1.5 rounded-full bg-purple-400"></span>
          <span class="text-xs text-neutral-400">Creator of Digital Vigital</span>
        </div>
        <h3 class="text-xl font-black text-white tracking-tight mt-0.5">
          SYN 606
        </h3>
        <p class="text-xs text-neutral-300 mt-1 leading-relaxed">
          Crafted with love for the Minecraft gaming community. Focused on clean system architecture, low-level Rust systems programming, and high-FPS gaming.
        </p>
      </div>
    </div>

    <button
      onclick={() => openLink("https://syn606.wtf")}
      class="px-5 py-2.5 rounded-xl bg-purple-600 hover:bg-purple-500 text-white font-bold text-xs uppercase tracking-wider flex items-center gap-2 transition-all cursor-pointer shadow-lg shadow-purple-600/30 shrink-0 hover:scale-105"
    >
      <span>Visit syn606.wtf</span>
      <ExternalLink class="w-4 h-4" />
    </button>
  </div>

  <!-- Project Philosophy & Why We Built DV -->
  <div class="p-8 rounded-3xl bg-[#0e1017]/90 border border-white/8 space-y-6 shadow-xl">
    <div>
      <h3 class="text-lg font-black uppercase text-white tracking-tight flex items-center gap-2.5">
        <Sparkles class="w-5 h-5 text-red-500" />
        <span>The Digital Vigital Manifesto</span>
      </h3>
      <p class="text-xs text-neutral-300 mt-1">
        Why we built an independent, completely free Minecraft launcher.
      </p>
    </div>

    <div class="text-sm text-neutral-300 leading-relaxed space-y-4 font-sans select-text">
      <p>
        Commercial Minecraft launchers have lost their way. Most existing launchers are heavy Electron wrappers that devour gigabytes of RAM in the background, force unwanted popups, run crypto-reward miners, and push commercial coin shops onto players before they even enter a game.
      </p>
      <p>
        We created <strong>Digital Vigital</strong> with a single unwavering goal: <strong>build the fastest, cleanest, most respectful desktop Minecraft launcher possible</strong>.
      </p>
    </div>

    <!-- Core Pillars Grid -->
    <div class="grid grid-cols-1 md:grid-cols-2 gap-4 pt-2">
      <!-- Pillar 1 -->
      <div class="p-5 rounded-2xl bg-white/3 border border-white/6 space-y-2">
        <div class="flex items-center gap-2 text-red-400 font-bold text-sm uppercase tracking-wide">
          <Zap class="w-4 h-4" />
          <span>Zero Bloat, Pure Rust Performance</span>
        </div>
        <p class="text-xs text-neutral-300 leading-relaxed">
          Built with Rust and Tauri v2. Starts in milliseconds, uses virtually zero background memory, and frees up 100% of your CPU and RAM for maximum Minecraft in-game FPS.
        </p>
      </div>

      <!-- Pillar 2 -->
      <div class="p-5 rounded-2xl bg-white/3 border border-white/6 space-y-2">
        <div class="flex items-center gap-2 text-emerald-400 font-bold text-sm uppercase tracking-wide">
          <HardDrive class="w-4 h-4" />
          <span>No C: Drive Pileup (Multi-Drive)</span>
        </div>
        <p class="text-xs text-neutral-300 leading-relaxed">
          Traditional launchers choke your main C: SSD with dozens of gigabytes of game assets, versions, and worlds. Digital Vigital lets you store your entire game library on any partition (D:, E:, etc.) with a single click.
        </p>
      </div>

      <!-- Pillar 3 -->
      <div class="p-5 rounded-2xl bg-white/3 border border-white/6 space-y-2">
        <div class="flex items-center gap-2 text-sky-400 font-bold text-sm uppercase tracking-wide">
          <ShieldCheck class="w-4 h-4" />
          <span>100% Free • No Ads • No Trackers</span>
        </div>
        <p class="text-xs text-neutral-300 leading-relaxed">
          No advertisements, no paywalls, and no telemetry selling your data. Direct official Microsoft OAuth logins with native encrypted platform keyring storage.
        </p>
      </div>

      <!-- Pillar 4 -->
      <div class="p-5 rounded-2xl bg-white/3 border border-white/6 space-y-2">
        <div class="flex items-center gap-2 text-amber-400 font-bold text-sm uppercase tracking-wide">
          <Coffee class="w-4 h-4" />
          <span>Multi-Java & Universal Loaders</span>
        </div>
        <p class="text-xs text-neutral-300 leading-relaxed">
          Dynamic automatic JVM resolution for Java 8 (legacy 1.8.9/1.12.2), Java 17, and Java 21, coupled with seamless support for Fabric, Forge, NeoForge, Quilt, and Modrinth/CurseForge catalogs.
        </p>
      </div>
    </div>
  </div>

  <!-- Official Server & Community Grid -->
  <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
    <!-- Card 1: Official Portal -->
    <div class="p-6 rounded-2xl bg-[#0e1017]/90 border border-white/8 space-y-3 flex flex-col justify-between">
      <div>
        <div class="flex items-center gap-2 text-red-400 text-xs font-bold uppercase tracking-wider mb-2">
          <Globe class="w-4 h-4" />
          <span>Official Portal</span>
        </div>
        <h4 class="text-base font-bold text-white">digitalvigital.fun/minecraft</h4>
        <p class="text-xs text-neutral-400 mt-1">
          The official home for Digital Vigital launcher updates, custom mods, SMP announcements, and guides.
        </p>
      </div>
      <button
        onclick={() => openLink("https://digitalvigital.fun/minecraft")}
        class="w-full px-4 py-2 rounded-xl bg-white/5 hover:bg-white/10 border border-white/10 text-neutral-200 hover:text-white font-bold text-xs flex items-center justify-center gap-2 transition-all cursor-pointer"
      >
        <span>Open Portal</span>
        <ExternalLink class="w-3.5 h-3.5" />
      </button>
    </div>

    <!-- Card 2: Official SMP -->
    <div class="p-6 rounded-2xl bg-[#0e1017]/90 border border-white/8 space-y-3 flex flex-col justify-between">
      <div>
        <div class="flex items-center gap-2 text-emerald-400 text-xs font-bold uppercase tracking-wider mb-2">
          <Server class="w-4 h-4" />
          <span>Digital Vigital SMP</span>
        </div>
        <h4 class="text-base font-bold text-white font-mono">play.digitalvigital.net</h4>
        <p class="text-xs text-neutral-400 mt-1">
          Join our official community Minecraft multiplayer server with custom performance optimizations and events.
        </p>
      </div>
      <button
        onclick={() => {
          navigator.clipboard.writeText("play.digitalvigital.net");
          launcher.showToast("Copied server IP: play.digitalvigital.net");
        }}
        class="w-full px-4 py-2 rounded-xl bg-emerald-500/15 hover:bg-emerald-500/25 border border-emerald-500/30 text-emerald-300 font-bold text-xs flex items-center justify-center gap-2 transition-all cursor-pointer"
      >
        <CheckCircle2 class="w-3.5 h-3.5" />
        <span>Copy Server IP</span>
      </button>
    </div>

    <!-- Card 3: GitHub Open Source -->
    <div class="p-6 rounded-2xl bg-[#0e1017]/90 border border-white/8 space-y-3 flex flex-col justify-between">
      <div>
        <div class="flex items-center gap-2 text-neutral-300 text-xs font-bold uppercase tracking-wider mb-2">
          <svg class="w-4 h-4 fill-current text-white" viewBox="0 0 24 24">
            <path d="M12 0C5.37 0 0 5.37 0 12c0 5.31 3.435 9.795 8.205 11.385.6.105.825-.255.825-.57 0-.285-.015-1.23-.015-2.235-3.015.555-3.795-.735-4.035-1.41-.135-.345-.72-1.41-1.23-1.695-.42-.225-1.02-.78-.015-.795.945-.015 1.62.87 1.845 1.23 1.08 1.815 2.805 1.305 3.495.99.105-.78.42-1.305.765-1.605-2.67-.3-5.46-1.335-5.46-5.925 0-1.305.465-2.385 1.23-3.225-.12-.3-.54-1.53.12-3.18 0 0 1.005-.315 3.3 1.23.96-.27 1.98-.405 3-.405s2.04.135 3 .405c2.295-1.56 3.3-1.23 3.3-1.23.66 1.65.24 2.88.12 3.18.765.84 1.23 1.905 1.23 3.225 0 4.605-2.805 5.625-5.475 5.925.435.375.81 1.095.81 2.22 0 1.605-.015 2.895-.015 3.3 0 .315.225.69.825.57A12.02 12.02 0 0024 12c0-6.63-5.37-12-12-12z"/>
          </svg>
          <span>Open Source</span>
        </div>
        <h4 class="text-base font-bold text-white">GitHub Repository</h4>
        <p class="text-xs text-neutral-400 mt-1">
          Browse the complete Rust codebase, report issues, contribute pull requests, and audit privacy.
        </p>
      </div>
      <button
        onclick={() => openLink("https://github.com/SYN606/dv-client")}
        class="w-full px-4 py-2 rounded-xl bg-white/5 hover:bg-white/10 border border-white/10 text-neutral-200 hover:text-white font-bold text-xs flex items-center justify-center gap-2 transition-all cursor-pointer"
      >
        <span>View on GitHub</span>
        <ExternalLink class="w-3.5 h-3.5" />
      </button>
    </div>
  </div>
</div>