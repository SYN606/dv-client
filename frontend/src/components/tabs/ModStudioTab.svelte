<script lang="ts">
  import { launcher } from "../../state/launcher.svelte";
  import { 
    Wrench, 
    Search, 
    FolderOpen, 
    RefreshCw, 
    Download, 
    Sparkles, 
    Zap, 
    ToggleLeft,
    ToggleRight
  } from "lucide-svelte";

  let searchQuery = $state("");

  function handleSearch() {
    launcher.searchModrinth(searchQuery);
  }

  function formatBytes(bytes: number) {
    if (bytes === 0) return "0 B";
    const k = 1024;
    const sizes = ["B", "KB", "MB", "GB"];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + " " + sizes[i];
  }
</script>

<div class="h-full overflow-y-auto p-8 space-y-6">
  <!-- Top Bar -->
  <div class="flex flex-col md:flex-row md:items-center justify-between gap-4 pb-4 border-b border-white/8">
    <div>
      <h2 class="text-2xl font-black text-white uppercase tracking-tight flex items-center gap-3">
        <Wrench class="w-6 h-6 text-red-500" />
        <span>Mod Studio & Optimizer</span>
      </h2>
      <p class="text-xs text-neutral-300 mt-1 font-medium">
        {#if launcher.activeInstance}
          Active Profile: <span class="text-white font-bold">{launcher.activeInstance.name}</span> ({launcher.activeInstance.mod_loader} {launcher.activeInstance.mc_version})
        {:else}
          Select a profile from the top bar to install and manage mods.
        {/if}
      </p>
    </div>

    <div class="flex items-center gap-2">
      <button
        onclick={() => launcher.loadInstalledMods()}
        title="Refresh local mods"
        class="px-4 py-2 rounded-xl bg-white/5 hover:bg-white/10 border border-white/10 text-neutral-200 hover:text-white text-xs font-bold flex items-center gap-2 transition-all cursor-pointer"
      >
        <RefreshCw class="w-3.5 h-3.5" />
        <span>Refresh</span>
      </button>

      <button
        onclick={() => launcher.openModsFolder()}
        title="Open mods folder"
        class="px-4 py-2 rounded-xl bg-white/5 hover:bg-white/10 border border-white/10 text-neutral-200 hover:text-white text-xs font-bold flex items-center gap-2 transition-all cursor-pointer"
      >
        <FolderOpen class="w-3.5 h-3.5" />
        <span>Open Mods Folder</span>
      </button>
    </div>
  </div>

  <!-- 1-Click Essential FPS Pack Banner -->
  <div class="relative rounded-2xl overflow-hidden border border-red-500/30 bg-gradient-to-r from-red-950/40 via-[#100a0e] to-[#0d0f17] p-6 shadow-xl flex flex-col md:flex-row items-center justify-between gap-4">
    <div class="flex items-center gap-4">
      <div class="w-12 h-12 rounded-xl bg-red-500/20 border border-red-500/40 flex items-center justify-center text-red-400 shrink-0">
        <Sparkles class="w-6 h-6" />
      </div>
      <div>
        <div class="flex items-center gap-2">
          <h3 class="font-bold text-base text-white">DV Essential Performance Suite</h3>
          <span class="px-2 py-0.5 rounded bg-red-500/20 text-red-400 text-xs uppercase font-bold">
            Recommended
          </span>
        </div>
        <p class="text-xs text-neutral-300 mt-1 leading-relaxed">
          One-click install Sodium, Lithium, and FerriteCore for maximum FPS rendering and smooth chunk generation.
        </p>
      </div>
    </div>

    <button
      onclick={() => launcher.installEssentialFpsPack()}
      disabled={!launcher.activeInstance || launcher.installingModId !== ""}
      class="w-full md:w-auto px-6 py-3 rounded-xl bg-red-600 hover:bg-red-500 text-white font-bold text-xs uppercase tracking-wider flex items-center justify-center gap-2 shadow-lg shadow-red-500/20 transition-all cursor-pointer shrink-0 disabled:opacity-50"
    >
      {#if launcher.installingModId}
        <div class="w-4 h-4 border-2 border-white/30 border-t-white rounded-full animate-spin"></div>
        <span>INSTALLING...</span>
      {:else}
        <Zap class="w-4 h-4" />
        <span>1-CLICK INSTALL ALL</span>
      {/if}
    </button>
  </div>

  <!-- Modrinth Online Search Section -->
  <div class="space-y-4">
    <div class="flex items-center justify-between">
      <h3 class="text-sm font-bold uppercase tracking-wider text-neutral-200 flex items-center gap-2">
        <Download class="w-4 h-4 text-emerald-400" />
        <span>Modrinth Mod Marketplace</span>
      </h3>
    </div>

    <div class="flex gap-2">
      <div class="relative flex-1">
        <Search class="w-4 h-4 text-neutral-400 absolute left-3.5 top-1/2 -translate-y-1/2" />
        <input
          type="text"
          bind:value={searchQuery}
          onkeydown={(e) => e.key === "Enter" && handleSearch()}
          placeholder="Search mods on Modrinth (e.g. Sodium, Iris, JourneyMap, JEI)..."
          class="w-full pl-10 pr-4 py-2.5 rounded-xl bg-white/5 border border-white/10 focus:border-red-500/50 focus:bg-white/8 outline-none text-xs text-white placeholder-neutral-400 transition-all"
        />
      </div>

      <button
        onclick={handleSearch}
        disabled={launcher.isSearchingModrinth}
        class="px-5 py-2.5 rounded-xl bg-white/6 hover:bg-white/12 border border-white/10 text-white font-bold text-xs uppercase tracking-wider transition-all cursor-pointer flex items-center gap-2 disabled:opacity-50"
      >
        {#if launcher.isSearchingModrinth}
          <div class="w-3.5 h-3.5 border-2 border-white/30 border-t-white rounded-full animate-spin"></div>
          <span>Searching...</span>
        {:else}
          <span>Search</span>
        {/if}
      </button>
    </div>

    <!-- Modrinth Results Grid -->
    {#if launcher.modrinthSearchResults.length > 0}
      <div class="grid grid-cols-1 md:grid-cols-2 gap-4 pt-2">
        {#each launcher.modrinthSearchResults as mod}
          <div class="p-4 rounded-xl bg-[#0e1017]/90 border border-white/8 hover:border-white/16 flex flex-col justify-between transition-all group">
            <div class="flex items-start gap-3.5">
              {#if mod.icon_url}
                <img 
                  src={mod.icon_url} 
                  alt={mod.title}
                  class="w-12 h-12 rounded-xl object-cover bg-black/40 border border-white/10 shrink-0"
                  onerror={(e) => { (e.currentTarget as HTMLElement).style.display = 'none'; }}
                />
              {:else}
                <div class="w-12 h-12 rounded-xl bg-red-500/10 border border-red-500/20 flex items-center justify-center text-red-400 shrink-0">
                  <Wrench class="w-6 h-6" />
                </div>
              {/if}

              <div class="flex-1 min-w-0">
                <div class="flex items-center justify-between">
                  <h4 class="font-bold text-sm text-white group-hover:text-red-400 transition-colors truncate">
                    {mod.title}
                  </h4>
                  <span class="text-xs text-neutral-400 shrink-0">
                    by {mod.author}
                  </span>
                </div>
                <p class="text-xs text-neutral-300 mt-1 line-clamp-2 leading-relaxed">
                  {mod.description}
                </p>
                <div class="flex items-center gap-3 mt-2 text-xs text-neutral-400">
                  <span>{(mod.downloads / 1000).toFixed(0)}k downloads</span>
                  {#if mod.categories && mod.categories.length > 0}
                    <span>•</span>
                    <span class="text-neutral-300">{mod.categories.slice(0, 2).join(", ")}</span>
                  {/if}
                </div>
              </div>
            </div>

            <button
              onclick={() => launcher.installModrinthMod(mod.project_id)}
              disabled={launcher.installingModId === mod.project_id}
              class="mt-3.5 w-full py-2 rounded-lg bg-white/5 hover:bg-red-500/20 hover:border-red-500/30 border border-white/10 text-white font-bold text-xs uppercase tracking-wider flex items-center justify-center gap-2 transition-all cursor-pointer disabled:opacity-50"
            >
              {#if launcher.installingModId === mod.project_id}
                <div class="w-3.5 h-3.5 border-2 border-white/30 border-t-white rounded-full animate-spin"></div>
                <span>Installing...</span>
              {:else}
                <Download class="w-3.5 h-3.5" />
                <span>Install Mod</span>
              {/if}
            </button>
          </div>
        {/each}
      </div>
    {/if}
  </div>

  <!-- Installed Mods List -->
  <div class="space-y-4 pt-4 border-t border-white/8">
    <div class="flex items-center justify-between">
      <h3 class="text-sm font-bold uppercase tracking-wider text-neutral-200 flex items-center gap-2">
        <Sparkles class="w-4 h-4 text-amber-400" />
        <span>Installed Mods ({launcher.installedMods.length})</span>
      </h3>
    </div>

    {#if launcher.installedMods.length > 0}
      <div class="space-y-2">
        {#each launcher.installedMods as mod}
          <div class="p-3.5 rounded-xl bg-[#0e1017]/80 border border-white/8 hover:border-white/16 flex items-center justify-between transition-all">
            <div class="flex items-center gap-3 min-w-0">
              <button
                onclick={() => launcher.toggleMod(mod)}
                title={mod.is_enabled ? "Disable Mod" : "Enable Mod"}
                class="transition-colors cursor-pointer text-neutral-400 hover:text-white"
              >
                {#if mod.is_enabled}
                  <ToggleRight class="w-6 h-6 text-red-500" />
                {:else}
                  <ToggleLeft class="w-6 h-6 text-neutral-500" />
                {/if}
              </button>

              <div class="min-w-0">
                <div class="text-sm font-bold text-white truncate {mod.is_enabled ? '' : 'text-neutral-500 line-through'}">
                  {mod.clean_name}
                </div>
                <div class="text-xs text-neutral-400 truncate font-mono">
                  {mod.filename}
                </div>
              </div>
            </div>

            <div class="flex items-center gap-3 shrink-0">
              <span class="text-xs font-mono text-neutral-300">
                {formatBytes(mod.size)}
              </span>
              <span class="text-xs uppercase px-2 py-0.5 rounded font-bold {mod.is_enabled ? 'bg-emerald-500/15 text-emerald-400 border border-emerald-500/25' : 'bg-neutral-500/15 text-neutral-400 border border-neutral-500/25'}">
                {mod.is_enabled ? "Enabled" : "Disabled"}
              </span>
            </div>
          </div>
        {/each}
      </div>
    {:else}
      <div class="p-8 text-center rounded-2xl border border-white/8 bg-white/2 space-y-2">
        <p class="text-xs text-neutral-300">No mods currently installed in this profile.</p>
        <p class="text-xs text-neutral-400">Use the 1-Click Essential FPS suite above or search Modrinth to add mods!</p>
      </div>
    {/if}
  </div>
</div>
