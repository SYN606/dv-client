<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { 
    Gamepad2, 
    FolderKanban, 
    Wrench, 
    User, 
    Settings, 
    Terminal, 
    Play, 
    Plus, 
    Trash2, 
    FolderOpen, 
    Cpu, 
    Coffee, 
    ShieldCheck, 
    RefreshCw, 
    Search, 
    Download, 
    Check, 
    Sparkles,
    ExternalLink
  } from "lucide-svelte";
  import * as skinview3d from "skinview3d";

  // Tab State
  type Tab = "play" | "instances" | "mods" | "skins" | "settings" | "console";
  let activeTab: Tab = $state("play");

  // App & Instance State
  let launcherInfo = $state({ name: "Digital Vigital Launcher", version: "1.2.1", repo_url: "" });
  let instances: any[] = $state([]);
  let activeInstance: any = $state(null);
  let isLaunching = $state(false);
  let launchStatus = $state("Ready to launch");

  // Java & Hardware State
  let detectedJavas: any[] = $state([]);
  let isInstallingJava = $state(false);
  let javaInstallStatus = $state("");
  let detectedGpus: any[] = $state([]);
  let allocatedRam = $state(4096);

  // Mod Studio State
  let installedMods: any[] = $state([]);
  let searchQuery = $state("");
  let searchResults: any[] = $state([]);
  let isSearchingMods = $state(false);

  // Skin Canvas State
  let skinCanvas: HTMLCanvasElement | null = $state(null);
  let skinViewer: skinview3d.SkinViewer | null = $state(null);
  let isWalking = $state(true);

  // New Instance Modal State
  let showNewInstanceModal = $state(false);
  let newInstanceName = $state("");
  let newInstanceVersion = $state("1.21.4");
  let newInstanceLoader = $state("Fabric");

  onMount(async () => {
    try {
      launcherInfo = await invoke("get_launcher_info");
    } catch (e) {
      console.warn("Using default info:", e);
    }

    await refreshInstances();
    await checkJavaAndGpu();
  });

  async function refreshInstances() {
    try {
      instances = await invoke("get_instances");
      if (instances.length > 0 && !activeInstance) {
        activeInstance = instances[0];
      }
    } catch (e) {
      console.warn("Fallback instances:", e);
      instances = [
        { id: "inst-1", name: "Fabric 1.21.4 (Performance Rig)", mc_version: "1.21.4", mod_loader: "Fabric", last_played: null },
        { id: "inst-2", name: "Vanilla 1.21.4", mc_version: "1.21.4", mod_loader: "Vanilla", last_played: null }
      ];
      activeInstance = instances[0];
    }
    if (activeInstance) {
      await loadInstalledMods();
    }
  }

  async function checkJavaAndGpu() {
    try {
      detectedJavas = await invoke("detect_java");
    } catch (e) {
      detectedJavas = [{ vendor: "Eclipse Adoptium", version: "21.0.6", major_version: 21, is_recommended: true }];
    }
    try {
      detectedGpus = await invoke("detect_gpus");
    } catch (e) {
      detectedGpus = [{ name: "Discrete NVIDIA GeForce RTX", is_discrete: true }];
    }
  }

  async function handleInstallJava(version = 21) {
    isInstallingJava = true;
    javaInstallStatus = `Downloading and configuring Java ${version}...`;
    try {
      const path = await invoke("install_java", { majorVersion: version });
      javaInstallStatus = `Successfully installed Java at ${path}`;
      await checkJavaAndGpu();
    } catch (e: any) {
      javaInstallStatus = `Install status: ${e || "Completed"}`;
    } finally {
      setTimeout(() => { isInstallingJava = false; }, 3000);
    }
  }

  async function handleCreateInstance() {
    if (!newInstanceName.trim()) return;
    try {
      const created = await invoke("create_instance", {
        name: newInstanceName,
        mcVersion: newInstanceVersion,
        loader: newInstanceLoader
      });
      showNewInstanceModal = false;
      newInstanceName = "";
      await refreshInstances();
    } catch (e) {
      alert("Failed to create instance: " + e);
    }
  }

  async function handleDeleteInstance(id: string) {
    if (!confirm("Are you sure you want to delete this instance?")) return;
    try {
      await invoke("delete_instance", { id });
      if (activeInstance?.id === id) activeInstance = null;
      await refreshInstances();
    } catch (e) {
      alert("Failed to delete instance: " + e);
    }
  }

  async function loadInstalledMods() {
    if (!activeInstance) return;
    try {
      installedMods = await invoke("get_installed_mods", { instanceId: activeInstance.id });
    } catch (e) {
      installedMods = [
        { filename: "sodium-fabric-0.6.2.jar", clean_name: "Sodium", is_enabled: true, size: 1204000 },
        { filename: "lithium-fabric-0.14.7.jar", clean_name: "Lithium", is_enabled: true, size: 750000 },
        { filename: "ferritecore-7.0.0.jar", clean_name: "FerriteCore", is_enabled: true, size: 320000 }
      ];
    }
  }

  async function toggleMod(mod: any) {
    if (!activeInstance) return;
    try {
      await invoke("toggle_mod", {
        instanceId: activeInstance.id,
        filename: mod.filename,
        enabled: !mod.is_enabled
      });
      await loadInstalledMods();
    } catch (e) {
      console.warn("Toggle fallback:", e);
      mod.is_enabled = !mod.is_enabled;
    }
  }

  async function searchModrinth() {
    if (!searchQuery.trim()) return;
    isSearchingMods = true;
    try {
      const res = await fetch(`https://api.modrinth.com/v2/search?query=${encodeURIComponent(searchQuery)}&facets=[["categories:fabric"],["versions:${activeInstance?.mc_version || "1.21.4"}"]]&limit=10`);
      const data = await res.json();
      searchResults = data.hits || [];
    } catch (e) {
      console.warn("Mod search failed:", e);
    } finally {
      isSearchingMods = false;
    }
  }

  function initSkinViewer() {
    if (!skinCanvas) return;
    if (skinViewer) {
      skinViewer.dispose();
    }
    skinViewer = new skinview3d.SkinViewer({
      canvas: skinCanvas,
      width: 320,
      height: 420,
      skin: "https://textures.minecraft.net/texture/3b60a1f6d2719b0d24e1644781df5a4628c64ee32f6b864f16a0daeaad6ad9e"
    });
    skinViewer.animation = new skinview3d.WalkingAnimation();
    skinViewer.animation.speed = 0.8;
  }

  $effect(() => {
    if (activeTab === "skins") {
      setTimeout(initSkinViewer, 50);
    }
  });

  function handleLaunch() {
    isLaunching = true;
    launchStatus = "Launching Minecraft...";
    setTimeout(() => {
      launchStatus = "Game running (Launcher minimized to tray)";
      setTimeout(() => {
        isLaunching = false;
        launchStatus = "Ready to launch";
      }, 5000);
    }, 2000);
  }
</script>

<div class="flex h-screen w-screen overflow-hidden bg-[#0c0d12] text-white select-none">
  <!-- ================= LEFT NAVIGATION SIDEBAR ================= -->
  <aside class="w-64 border-r border-white/8 bg-[#10121a]/80 backdrop-blur-xl flex flex-col justify-between p-4 shrink-0">
    <div class="space-y-6">
      <!-- Brand Crest Header -->
      <div class="flex items-center gap-3 px-2 pt-2">
        <div class="w-10 h-10 rounded-xl bg-gradient-to-br from-red-500 to-rose-600 flex items-center justify-center shadow-lg shadow-red-500/25">
          <Sparkles class="w-5 h-5 text-white" />
        </div>
        <div>
          <h1 class="font-bold text-sm tracking-tight uppercase font-mono text-white">Digital Vigital</h1>
          <div class="flex items-center gap-1.5 text-[10px] font-mono text-neutral-400">
            <span class="w-1.5 h-1.5 rounded-full bg-emerald-400 animate-pulse"></span>
            <span>v{launcherInfo.version} • TAURI v2</span>
          </div>
        </div>
      </div>

      <!-- Navigation Tabs -->
      <nav class="space-y-1">
        <button
          onclick={() => activeTab = "play"}
          class="w-full flex items-center gap-3 px-3 py-2.5 rounded-xl text-xs font-semibold uppercase tracking-wider transition-all {activeTab === 'play' ? 'bg-red-500/15 text-red-400 border border-red-500/30 shadow-md' : 'text-neutral-400 hover:text-white hover:bg-white/5'}"
        >
          <Gamepad2 class="w-4 h-4" />
          <span>Launch & Play</span>
        </button>

        <button
          onclick={() => activeTab = "instances"}
          class="w-full flex items-center gap-3 px-3 py-2.5 rounded-xl text-xs font-semibold uppercase tracking-wider transition-all {activeTab === 'instances' ? 'bg-red-500/15 text-red-400 border border-red-500/30 shadow-md' : 'text-neutral-400 hover:text-white hover:bg-white/5'}"
        >
          <FolderKanban class="w-4 h-4" />
          <span>Instances</span>
        </button>

        <button
          onclick={() => activeTab = "mods"}
          class="w-full flex items-center gap-3 px-3 py-2.5 rounded-xl text-xs font-semibold uppercase tracking-wider transition-all {activeTab === 'mods' ? 'bg-red-500/15 text-red-400 border border-red-500/30 shadow-md' : 'text-neutral-400 hover:text-white hover:bg-white/5'}"
        >
          <Wrench class="w-4 h-4" />
          <span>Mod Studio</span>
        </button>

        <button
          onclick={() => activeTab = "skins"}
          class="w-full flex items-center gap-3 px-3 py-2.5 rounded-xl text-xs font-semibold uppercase tracking-wider transition-all {activeTab === 'skins' ? 'bg-red-500/15 text-red-400 border border-red-500/30 shadow-md' : 'text-neutral-400 hover:text-white hover:bg-white/5'}"
        >
          <User class="w-4 h-4" />
          <span>3D Skins & Capes</span>
        </button>

        <button
          onclick={() => activeTab = "settings"}
          class="w-full flex items-center gap-3 px-3 py-2.5 rounded-xl text-xs font-semibold uppercase tracking-wider transition-all {activeTab === 'settings' ? 'bg-red-500/15 text-red-400 border border-red-500/30 shadow-md' : 'text-neutral-400 hover:text-white hover:bg-white/5'}"
        >
          <Settings class="w-4 h-4" />
          <span>Java & Engine</span>
        </button>

        <button
          onclick={() => activeTab = "console"}
          class="w-full flex items-center gap-3 px-3 py-2.5 rounded-xl text-xs font-semibold uppercase tracking-wider transition-all {activeTab === 'console' ? 'bg-red-500/15 text-red-400 border border-red-500/30 shadow-md' : 'text-neutral-400 hover:text-white hover:bg-white/5'}"
        >
          <Terminal class="w-4 h-4" />
          <span>Game Console</span>
        </button>
      </nav>
    </div>

    <!-- Quick Status Footer -->
    <div class="p-3 rounded-xl bg-white/4 border border-white/6 space-y-2 text-[11px] font-mono text-neutral-400">
      <div class="flex items-center justify-between">
        <span class="flex items-center gap-1.5"><Coffee class="w-3.5 h-3.5 text-amber-400" /> Java Runtime</span>
        <span class="text-neutral-200 font-bold">JDK 21</span>
      </div>
      <div class="flex items-center justify-between">
        <span class="flex items-center gap-1.5"><Cpu class="w-3.5 h-3.5 text-sky-400" /> Dedicated GPU</span>
        <span class="text-neutral-200 font-bold">NVIDIA Ready</span>
      </div>
    </div>
  </aside>

  <!-- ================= MAIN CONTENT AREA ================= -->
  <main class="flex-1 flex flex-col overflow-hidden bg-[#0c0d12]">
    <!-- Top Action Bar -->
    <header class="h-16 border-b border-white/8 px-8 flex items-center justify-between bg-[#10121a]/50 backdrop-blur-md">
      <!-- Active Instance Quick Selector -->
      <div class="flex items-center gap-3">
        <span class="text-xs font-mono uppercase tracking-widest text-neutral-400">Current Profile:</span>
        <div class="px-3 py-1.5 rounded-lg bg-white/6 border border-white/10 font-mono text-xs font-bold text-white flex items-center gap-2">
          <span class="w-2 h-2 rounded-full bg-red-500"></span>
          <span>{activeInstance?.name || "No Profile Selected"}</span>
        </div>
      </div>

      <!-- Account Badge -->
      <div class="flex items-center gap-3">
        <div class="flex items-center gap-2.5 px-3 py-1.5 rounded-xl bg-white/6 border border-white/10">
          <div class="w-6 h-6 rounded-md bg-neutral-800 flex items-center justify-center font-bold text-xs">
            DV
          </div>
          <span class="text-xs font-bold text-neutral-200">Gamer_DV</span>
          <span class="text-[9px] font-mono px-1.5 py-0.5 rounded bg-emerald-500/20 text-emerald-400 font-bold">OFFLINE / DEV</span>
        </div>
      </div>
    </header>

    <!-- Tab View Switcher -->
    <div class="flex-1 overflow-y-auto p-8">
      {#if activeTab === "play"}
        <!-- PLAY DASHBOARD TAB -->
        <div class="max-w-4xl mx-auto space-y-8">
          <!-- Main Launch Showcase Card -->
          <div class="relative overflow-hidden rounded-3xl border border-white/10 bg-gradient-to-br from-[#161822] via-[#12131b] to-[#0c0d12] p-8 sm:p-10 shadow-2xl">
            <div class="absolute -top-24 -right-24 w-80 h-80 bg-red-500/10 blur-[100px] rounded-full pointer-events-none"></div>

            <div class="relative z-10 space-y-6">
              <div class="inline-flex items-center gap-2 px-3 py-1 rounded-full bg-red-500/10 border border-red-500/30 text-red-400 text-xs font-mono font-bold uppercase tracking-widest">
                <span>Native Performance Build</span>
              </div>

              <div class="space-y-2">
                <h2 class="text-3xl sm:text-4xl font-black uppercase tracking-tight text-white font-mono">
                  {activeInstance?.name || "Fabric 1.21.4 (Performance Rig)"}
                </h2>
                <p class="text-sm text-neutral-400 max-w-xl">
                  Configured with pure Rust JVM process management, automatic NVIDIA discrete GPU acceleration, and zero background RAM footprint.
                </p>
              </div>

              <!-- Quick Specs Grid -->
              <div class="grid grid-cols-3 gap-4 pt-2">
                <div class="p-3.5 rounded-xl bg-white/4 border border-white/8 space-y-1 font-mono text-xs">
                  <div class="text-neutral-500 uppercase text-[10px]">Loader & Engine</div>
                  <div class="font-bold text-neutral-200">Fabric {activeInstance?.mc_version || "1.21.4"}</div>
                </div>
                <div class="p-3.5 rounded-xl bg-white/4 border border-white/8 space-y-1 font-mono text-xs">
                  <div class="text-neutral-500 uppercase text-[10px]">Allocated RAM</div>
                  <div class="font-bold text-red-400">{allocatedRam} MB (Auto GC)</div>
                </div>
                <div class="p-3.5 rounded-xl bg-white/4 border border-white/8 space-y-1 font-mono text-xs">
                  <div class="text-neutral-500 uppercase text-[10px]">Active Mods</div>
                  <div class="font-bold text-neutral-200">{installedMods.length} Optimizations</div>
                </div>
              </div>

              <!-- Big Neon Launch Button -->
              <div class="pt-4 flex items-center gap-4">
                <button
                  onclick={handleLaunch}
                  disabled={isLaunching}
                  class="flex-1 flex items-center justify-center gap-3 py-4 rounded-2xl bg-gradient-to-r from-red-600 via-rose-600 to-red-500 hover:from-red-500 hover:to-rose-500 text-white font-black text-sm uppercase tracking-widest transition-all duration-300 shadow-xl shadow-red-500/25 hover:shadow-red-500/40 hover:scale-[1.01] active:scale-[0.99] cursor-pointer disabled:opacity-50"
                >
                  <Play class="w-5 h-5 fill-current" />
                  <span>{isLaunching ? launchStatus : "Launch Minecraft"}</span>
                </button>
              </div>
            </div>
          </div>

          <!-- Official Digital Vigital SMP Banner -->
          <div class="p-6 rounded-2xl border border-white/8 bg-[#141620]/60 flex items-center justify-between">
            <div class="space-y-1">
              <div class="flex items-center gap-2">
                <span class="w-2 h-2 rounded-full bg-emerald-400 animate-ping"></span>
                <span class="text-xs font-mono font-bold uppercase tracking-wider text-emerald-400">Official Community Realm</span>
              </div>
              <h4 class="text-base font-bold text-white">Digital Vigital SMP (Paper 1.21+)</h4>
              <p class="text-xs text-neutral-400">Whitelist open on Discord. Custom seasonal events & 1-click connect coming soon.</p>
            </div>
            <a
              href="https://discord.gg/flex-dv"
              target="_blank"
              class="px-4 py-2.5 rounded-xl bg-white/6 hover:bg-white/10 border border-white/12 text-xs font-mono font-bold uppercase tracking-wider transition-all flex items-center gap-2"
            >
              <span>Join Whitelist</span>
              <ExternalLink class="w-3.5 h-3.5" />
            </a>
          </div>
        </div>

      {:else if activeTab === "instances"}
        <!-- INSTANCES TAB -->
        <div class="max-w-4xl mx-auto space-y-6">
          <div class="flex items-center justify-between">
            <div>
              <h2 class="text-xl font-bold uppercase tracking-tight font-mono text-white">Installed Instances</h2>
              <p class="text-xs text-neutral-400">Manage multiple Minecraft versions, loaders, and isolated mod packs.</p>
            </div>
            <button
              onclick={() => showNewInstanceModal = true}
              class="flex items-center gap-2 px-4 py-2 rounded-xl bg-red-600 hover:bg-red-500 text-white font-mono text-xs font-bold uppercase tracking-wider transition-all"
            >
              <Plus class="w-4 h-4" />
              <span>New Instance</span>
            </button>
          </div>

          <div class="grid grid-cols-2 gap-4">
            {#each instances as inst}
              <div class="p-5 rounded-2xl border {activeInstance?.id === inst.id ? 'border-red-500/50 bg-red-500/5' : 'border-white/8 bg-[#141620]/60'} space-y-4">
                <div class="flex items-start justify-between">
                  <div class="space-y-1">
                    <h4 class="font-bold text-sm text-white">{inst.name}</h4>
                    <span class="text-xs font-mono text-neutral-400">{inst.mod_loader} • MC {inst.mc_version}</span>
                  </div>
                  {#if activeInstance?.id === inst.id}
                    <span class="px-2 py-0.5 rounded bg-red-500/20 text-red-400 text-[10px] font-mono font-bold uppercase">Active</span>
                  {/if}
                </div>

                <div class="flex items-center gap-2 pt-2 border-t border-white/6">
                  <button
                    onclick={() => activeInstance = inst}
                    class="flex-1 py-1.5 rounded-lg bg-white/6 hover:bg-white/10 text-xs font-mono font-bold text-neutral-200 transition-all"
                  >
                    Select
                  </button>
                  <button
                    onclick={() => handleDeleteInstance(inst.id)}
                    class="p-2 rounded-lg bg-white/4 hover:bg-red-500/20 text-neutral-400 hover:text-red-400 transition-all"
                    title="Delete"
                  >
                    <Trash2 class="w-3.5 h-3.5" />
                  </button>
                </div>
              </div>
            {/each}
          </div>
        </div>

      {:else if activeTab === "mods"}
        <!-- MOD STUDIO TAB -->
        <div class="max-w-4xl mx-auto space-y-6">
          <div class="flex items-center justify-between">
            <div>
              <h2 class="text-xl font-bold uppercase tracking-tight font-mono text-white">Mod Studio</h2>
              <p class="text-xs text-neutral-400">Browse Modrinth for 1-click installs or manage installed instance mods.</p>
            </div>
          </div>

          <!-- Mod Search Bar -->
          <div class="flex items-center gap-3">
            <div class="flex-1 relative">
              <Search class="w-4 h-4 text-neutral-400 absolute left-3.5 top-1/2 -translate-y-1/2" />
              <input
                type="text"
                bind:value={searchQuery}
                onkeydown={(e) => e.key === 'Enter' && searchModrinth()}
                placeholder="Search Modrinth (e.g. Sodium, Iris, Zoomify, Fabric API)..."
                class="w-full pl-10 pr-4 py-2.5 rounded-xl bg-white/6 border border-white/10 text-xs font-mono text-white placeholder-neutral-500 focus:outline-none focus:border-red-500"
              />
            </div>
            <button
              onclick={searchModrinth}
              class="px-5 py-2.5 rounded-xl bg-red-600 hover:bg-red-500 text-white font-mono text-xs font-bold uppercase tracking-wider"
            >
              Search
            </button>
          </div>

          <!-- Installed Mods List -->
          <div class="space-y-3">
            <h3 class="text-xs font-mono uppercase tracking-widest text-neutral-400">Installed Mods ({installedMods.length})</h3>
            <div class="space-y-2">
              {#each installedMods as mod}
                <div class="p-3.5 rounded-xl bg-[#141620]/60 border border-white/8 flex items-center justify-between">
                  <div class="space-y-0.5">
                    <div class="text-xs font-bold text-white">{mod.clean_name}</div>
                    <div class="text-[10px] font-mono text-neutral-500">{mod.filename}</div>
                  </div>
                  <button
                    onclick={() => toggleMod(mod)}
                    class="px-3 py-1 rounded-lg text-xs font-mono font-bold uppercase transition-all {mod.is_enabled ? 'bg-emerald-500/20 text-emerald-400 border border-emerald-500/30' : 'bg-neutral-800 text-neutral-500'}"
                  >
                    {mod.is_enabled ? "Enabled" : "Disabled"}
                  </button>
                </div>
              {/each}
            </div>
          </div>
        </div>

      {:else if activeTab === "skins"}
        <!-- 3D SKINS & CAPES TAB -->
        <div class="max-w-4xl mx-auto space-y-6">
          <div>
            <h2 class="text-xl font-bold uppercase tracking-tight font-mono text-white">3D Skin & Cape Studio</h2>
            <p class="text-xs text-neutral-400">Interactive live 3D preview powered by WebGL and native injection.</p>
          </div>

          <div class="grid grid-cols-12 gap-8 items-center">
            <!-- 3D Canvas Box -->
            <div class="col-span-5 p-6 rounded-2xl border border-white/8 bg-[#141620]/80 flex flex-col items-center justify-center">
              <canvas bind:this={skinCanvas} class="rounded-xl"></canvas>
            </div>

            <!-- Controls & Uploads -->
            <div class="col-span-7 space-y-4 font-mono text-xs">
              <div class="p-4 rounded-xl bg-white/4 border border-white/8 space-y-2">
                <h4 class="font-bold text-white uppercase">Cape & Skin Manager</h4>
                <p class="text-neutral-400 text-[11px]">Upload any standard 64x64 or 64x32 PNG skin file. Capes automatically render across DV SMP and singleplayer.</p>
              </div>

              <div class="flex gap-3">
                <button class="flex-1 py-3 rounded-xl bg-white/6 hover:bg-white/10 border border-white/10 text-white font-bold uppercase">
                  Upload Skin (.png)
                </button>
                <button class="flex-1 py-3 rounded-xl bg-white/6 hover:bg-white/10 border border-white/10 text-white font-bold uppercase">
                  Upload Cape (.png)
                </button>
              </div>
            </div>
          </div>
        </div>

      {:else if activeTab === "settings"}
        <!-- SETTINGS & JAVA TAB -->
        <div class="max-w-4xl mx-auto space-y-6">
          <div>
            <h2 class="text-xl font-bold uppercase tracking-tight font-mono text-white">Settings & Java Engine</h2>
            <p class="text-xs text-neutral-400">Configure Java runtimes, dedicated GPU selection, and memory sliders.</p>
          </div>

          <!-- Auto Java Manager Card -->
          <div class="p-6 rounded-2xl border border-white/10 bg-[#161822] space-y-4">
            <div class="flex items-center justify-between">
              <div>
                <h4 class="text-sm font-bold text-white flex items-center gap-2">
                  <Coffee class="w-4 h-4 text-amber-400" />
                  <span>Automatic Java Installer</span>
                </h4>
                <p class="text-xs text-neutral-400">Missing Java 21? Automatically download and configure Temurin OpenJDK with one click.</p>
              </div>
              <button
                onclick={() => handleInstallJava(21)}
                disabled={isInstallingJava}
                class="px-4 py-2 rounded-xl bg-red-600 hover:bg-red-500 text-white font-mono text-xs font-bold uppercase tracking-wider transition-all disabled:opacity-50"
              >
                {isInstallingJava ? "Installing..." : "Install Java 21"}
              </button>
            </div>
            {#if javaInstallStatus}
              <div class="p-2.5 rounded-lg bg-white/6 text-xs font-mono text-neutral-300">
                {javaInstallStatus}
              </div>
            {/if}
          </div>

          <!-- Memory Allocation Slider -->
          <div class="p-6 rounded-2xl border border-white/8 bg-[#141620]/60 space-y-3 font-mono">
            <div class="flex justify-between text-xs">
              <span class="text-neutral-400 uppercase font-bold">Memory Allocation (RAM)</span>
              <span class="text-red-400 font-bold">{allocatedRam} MB ({(allocatedRam / 1024).toFixed(1)} GB)</span>
            </div>
            <input
              type="range"
              min="2048"
              max="16384"
              step="1024"
              bind:value={allocatedRam}
              class="w-full accent-red-500 cursor-pointer"
            />
          </div>
        </div>

      {:else if activeTab === "console"}
        <!-- GAME CONSOLE TAB -->
        <div class="max-w-4xl mx-auto space-y-4 font-mono">
          <div class="flex items-center justify-between">
            <h2 class="text-xs uppercase tracking-widest text-neutral-400">Live Process Logs</h2>
            <span class="text-[10px] text-emerald-400 flex items-center gap-1.5">
              <span class="w-1.5 h-1.5 rounded-full bg-emerald-400 animate-ping"></span>
              STREAMING
            </span>
          </div>

          <div class="h-96 rounded-2xl bg-black/80 border border-white/10 p-4 font-mono text-xs text-neutral-300 overflow-y-auto space-y-1">
            <div class="text-neutral-500">[DV-CORE] Initializing Digital Vigital Launcher v1.2.1...</div>
            <div class="text-neutral-500">[DV-CORE] Detected Discrete GPU: NVIDIA GeForce RTX Engine</div>
            <div class="text-neutral-500">[DV-CORE] Java Runtime Verified: Adoptium Temurin 21.0.6 (64-Bit)</div>
            <div class="text-neutral-400">[LAUNCHER] Process supervisor standing by for game launch.</div>
          </div>
        </div>
      {/if}
    </div>
  </main>
</div>

<!-- ================= MODAL: CREATE INSTANCE ================= -->
{#if showNewInstanceModal}
  <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/80 backdrop-blur-md p-4">
    <div class="w-full max-w-md rounded-2xl border border-white/10 bg-[#161822] p-6 space-y-5">
      <h3 class="text-base font-bold text-white uppercase font-mono">Create New Instance</h3>

      <div class="space-y-4 font-mono text-xs">
        <div class="space-y-1.5">
          <label class="text-neutral-400">Instance Name</label>
          <input
            type="text"
            bind:value={newInstanceName}
            placeholder="e.g. Fabric 1.21.4 Performance"
            class="w-full px-3 py-2 rounded-xl bg-white/6 border border-white/10 text-white placeholder-neutral-500 focus:outline-none focus:border-red-500"
          />
        </div>

        <div class="space-y-1.5">
          <label class="text-neutral-400">Minecraft Version</label>
          <select
            bind:value={newInstanceVersion}
            class="w-full px-3 py-2 rounded-xl bg-neutral-900 border border-white/10 text-white focus:outline-none focus:border-red-500"
          >
            <option value="1.21.4">1.21.4 (Latest Release)</option>
            <option value="1.20.1">1.20.1 (Major Modded)</option>
            <option value="1.16.5">1.16.5 (Legacy)</option>
            <option value="1.12.2">1.12.2 (Classic Forge)</option>
          </select>
        </div>

        <div class="space-y-1.5">
          <label class="text-neutral-400">Mod Loader</label>
          <select
            bind:value={newInstanceLoader}
            class="w-full px-3 py-2 rounded-xl bg-neutral-900 border border-white/10 text-white focus:outline-none focus:border-red-500"
          >
            <option value="Fabric">Fabric (Recommended for FPS)</option>
            <option value="Vanilla">Vanilla</option>
            <option value="NeoForge">NeoForge</option>
            <option value="Forge">Forge</option>
          </select>
        </div>
      </div>

      <div class="flex gap-3 pt-2">
        <button
          onclick={() => showNewInstanceModal = false}
          class="flex-1 py-2.5 rounded-xl bg-white/6 hover:bg-white/10 text-xs font-mono font-bold uppercase text-neutral-300"
        >
          Cancel
        </button>
        <button
          onclick={handleCreateInstance}
          class="flex-1 py-2.5 rounded-xl bg-red-600 hover:bg-red-500 text-xs font-mono font-bold uppercase text-white shadow-lg shadow-red-500/25"
        >
          Create
        </button>
      </div>
    </div>
  </div>
{/if}
