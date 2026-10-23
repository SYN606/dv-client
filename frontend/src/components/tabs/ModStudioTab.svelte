<script lang="ts">
  import { onMount } from "svelte";
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
    ToggleRight, 
    Trash2, 
    Check, 
    ArrowUpDown, 
    SlidersHorizontal, 
    Box, 
    FileUp, 
    Layers, 
    Compass, 
    AlertTriangle, 
    CheckCheck, 
    XCircle, 
    X 
  } from "lucide-svelte";

  const categories = [
    { id: "all", label: "All Mods" },
    { id: "optimization", label: "Optimization" },
    { id: "utility", label: "Utility" },
    { id: "decoration", label: "Decoration" },
    { id: "technology", label: "Technology" },
    { id: "equipment", label: "Equipment" },
    { id: "adventure", label: "Adventure" },
    { id: "library", label: "Library" },
  ];

  const sortOptions = [
    { id: "downloads", label: "Most Popular" },
    { id: "follows", label: "Most Followed" },
    { id: "updated", label: "Recently Updated" },
    { id: "newest", label: "Newest" },
  ];

  onMount(() => {
    if (launcher.activeInstance) {
      launcher.loadInstalledMods();
      if (launcher.modrinthSearchResults.length === 0) {
        launcher.fetchModsForSelectedProfile();
      }
    }
  });

  function handleSearch() {
    launcher.searchModrinth(launcher.modrinthQuery);
  }

  function handleCategoryClick(catId: string) {
    launcher.fetchModsForSelectedProfile(catId, undefined, launcher.modrinthQuery);
  }

  function handleSortChange(e: Event) {
    const target = e.target as HTMLSelectElement;
    launcher.fetchModsForSelectedProfile(undefined, target.value, launcher.modrinthQuery);
  }

  function handleRefresh() {
    launcher.loadInstalledMods();
    launcher.fetchModsForSelectedProfile(launcher.modrinthCategory, launcher.modrinthSortBy, launcher.modrinthQuery);
  }

  // Reactive counts and filters for Installed Mods
  let enabledInstalledCount = $derived(
    launcher.installedMods.filter(m => m.is_enabled).length
  );
  let disabledInstalledCount = $derived(
    launcher.installedMods.filter(m => !m.is_enabled).length
  );

  let displayedInstalledMods = $derived.by(() => {
    let list = launcher.installedMods;
    if (launcher.installedModsFilter === "enabled") {
      list = list.filter(m => m.is_enabled);
    } else if (launcher.installedModsFilter === "disabled") {
      list = list.filter(m => !m.is_enabled);
    }
    if (launcher.installedModsSearch.trim()) {
      const q = launcher.installedModsSearch.toLowerCase();
      list = list.filter(m => 
        (m.name && m.name.toLowerCase().includes(q)) ||
        (m.file_name && m.file_name.toLowerCase().includes(q)) ||
        (m.description && m.description.toLowerCase().includes(q)) ||
        (m.author && m.author.toLowerCase().includes(q))
      );
    }
    return list;
  });
</script>

<div class="h-full overflow-y-auto p-8 space-y-6">
  <!-- Top Header & View Switcher Bar -->
  <div class="flex flex-col md:flex-row md:items-center justify-between gap-4 pb-4 border-b border-white/8">
    <div>
      <div class="flex items-center gap-3">
        <div class="w-10 h-10 rounded-xl bg-red-500/10 border border-red-500/20 flex items-center justify-center text-red-500 shrink-0">
          <Wrench class="w-5 h-5" />
        </div>
        <div>
          <h2 class="text-2xl font-black text-white uppercase tracking-tight">
            Mod Studio
          </h2>
          <p class="text-xs text-neutral-300 font-medium">
            {#if launcher.activeInstance}
              Profile: <span class="text-white font-bold">{launcher.activeInstance.name}</span> 
              <span class="px-2 py-0.5 ml-2 rounded bg-red-500/15 text-red-400 border border-red-500/25 uppercase font-mono font-bold text-[11px]">
                {launcher.activeInstance.mod_loader} {launcher.activeInstance.mc_version}
              </span>
            {:else}
              <span class="text-amber-400 font-semibold">Select a profile to manage and install mods.</span>
            {/if}
          </p>
        </div>
      </div>
    </div>

    <!-- Segmented Two-Part View Switcher -->
    <div class="flex items-center gap-2 flex-wrap">
      <div class="flex items-center p-1 bg-white/4 border border-white/8 rounded-xl shadow-inner">
        <button
          onclick={() => launcher.modStudioSubTab = "installed"}
          class="flex items-center gap-2 px-4 py-2 rounded-lg font-bold text-xs uppercase tracking-wider transition-all cursor-pointer {launcher.modStudioSubTab === 'installed' ? 'bg-red-600 text-white shadow-md shadow-red-600/30' : 'text-neutral-400 hover:text-white'}"
        >
          <Layers class="w-3.5 h-3.5" />
          <span>Installed Mods</span>
          <span class="px-1.5 py-0.5 rounded-full text-[10px] font-mono {launcher.modStudioSubTab === 'installed' ? 'bg-black/30 text-white' : 'bg-white/10 text-neutral-300'}">
            {launcher.installedMods.length}
          </span>
        </button>

        <button
          onclick={() => launcher.modStudioSubTab = "catalog"}
          class="flex items-center gap-2 px-4 py-2 rounded-lg font-bold text-xs uppercase tracking-wider transition-all cursor-pointer {launcher.modStudioSubTab === 'catalog' ? 'bg-red-600 text-white shadow-md shadow-red-600/30' : 'text-neutral-400 hover:text-white'}"
        >
          <Compass class="w-3.5 h-3.5" />
          <span>Mod Catalog</span>
          <span class="px-1.5 py-0.5 rounded-full text-[10px] font-mono {launcher.modStudioSubTab === 'catalog' ? 'bg-black/30 text-white' : 'bg-white/10 text-neutral-300'}">
            {launcher.modrinthSearchResults.length}
          </span>
        </button>
      </div>

      <button
        onclick={handleRefresh}
        title="Refresh mods"
        class="p-2.5 rounded-xl bg-white/5 hover:bg-white/10 border border-white/10 text-neutral-200 hover:text-white transition-all cursor-pointer"
      >
        <RefreshCw class="w-4 h-4 {launcher.isSearchingModrinth ? 'animate-spin' : ''}" />
      </button>

      <button
        onclick={() => launcher.openModsFolder()}
        title="Open mods folder in Windows Explorer"
        class="px-3.5 py-2 rounded-xl bg-white/5 hover:bg-white/10 border border-white/10 text-neutral-200 hover:text-white text-xs font-bold flex items-center gap-2 transition-all cursor-pointer"
      >
        <FolderOpen class="w-3.5 h-3.5" />
        <span class="hidden sm:inline">Mods Folder</span>
      </button>
    </div>
  </div>

  <!-- Vanilla Profile Notice (If active instance is Vanilla) -->
  {#if launcher.activeInstance && launcher.activeInstance.mod_loader.toLowerCase() === "vanilla"}
    <div class="p-4 rounded-xl border border-amber-500/30 bg-amber-950/30 flex flex-col sm:flex-row items-start sm:items-center justify-between gap-3 text-amber-200">
      <div class="flex items-center gap-3">
        <AlertTriangle class="w-5 h-5 text-amber-400 shrink-0" />
        <div class="text-xs">
          <strong class="text-white block font-bold text-sm">Vanilla Profile Selected</strong>
          Vanilla Minecraft ignores mods. Switch to a Fabric, Forge, or NeoForge profile in the Profiles tab to activate modding capabilities.
        </div>
      </div>
      <button
        onclick={() => launcher.activeTab = "instances"}
        class="px-4 py-2 rounded-xl bg-amber-500 hover:bg-amber-400 text-black font-bold text-xs uppercase tracking-wider transition-all cursor-pointer shrink-0"
      >
        Switch Profile
      </button>
    </div>
  {/if}

  <!-- ========================================================================= -->
  <!-- PART 1: INSTALLED MODS VIEW                                               -->
  <!-- ========================================================================= -->
  {#if launcher.modStudioSubTab === "installed"}
    <div class="space-y-4">
      <!-- Controls Toolbar -->
      <div class="flex flex-col sm:flex-row items-stretch sm:items-center justify-between gap-3 bg-[#0d0e14]/70 border border-white/8 p-3 rounded-2xl">
        <!-- Live Search Filter -->
        <div class="relative flex-1">
          <Search class="w-3.5 h-3.5 text-neutral-400 absolute left-3 top-1/2 -translate-y-1/2" />
          <input
            type="text"
            bind:value={launcher.installedModsSearch}
            placeholder="Search installed mods by name, file, or author..."
            class="w-full pl-8 pr-7 py-1.5 rounded-xl bg-white/5 border border-white/10 focus:border-red-500/50 outline-none text-xs text-white placeholder-neutral-400 transition-all font-mono"
          />
          {#if launcher.installedModsSearch}
            <button
              onclick={() => launcher.installedModsSearch = ""}
              class="absolute right-2.5 top-1/2 -translate-y-1/2 text-neutral-400 hover:text-white cursor-pointer"
            >
              <X class="w-3.5 h-3.5" />
            </button>
          {/if}
        </div>

        <!-- Filter Status Pills -->
        <div class="flex items-center gap-1.5">
          <button
            onclick={() => launcher.installedModsFilter = "all"}
            class="px-2.5 py-1 rounded-lg text-xs font-bold transition-all cursor-pointer {launcher.installedModsFilter === 'all' ? 'bg-white/15 text-white border border-white/25' : 'bg-white/4 text-neutral-400 hover:text-white'}"
          >
            All ({launcher.installedMods.length})
          </button>
          <button
            onclick={() => launcher.installedModsFilter = "enabled"}
            class="px-2.5 py-1 rounded-lg text-xs font-bold transition-all cursor-pointer {launcher.installedModsFilter === 'enabled' ? 'bg-emerald-500/20 text-emerald-300 border border-emerald-500/30' : 'bg-white/4 text-neutral-400 hover:text-emerald-300'}"
          >
            Enabled ({enabledInstalledCount})
          </button>
          <button
            onclick={() => launcher.installedModsFilter = "disabled"}
            class="px-2.5 py-1 rounded-lg text-xs font-bold transition-all cursor-pointer {launcher.installedModsFilter === 'disabled' ? 'bg-neutral-500/20 text-neutral-300 border border-neutral-500/30' : 'bg-white/4 text-neutral-400 hover:text-neutral-300'}"
          >
            Disabled ({disabledInstalledCount})
          </button>
        </div>

        <!-- Bulk Actions & Import -->
        <div class="flex items-center gap-2 border-t sm:border-t-0 sm:border-l border-white/8 pt-2 sm:pt-0 sm:pl-3">
          <button
            onclick={() => launcher.bulkToggleMods(true)}
            title="Enable all mods"
            class="p-2 rounded-xl bg-white/5 hover:bg-emerald-500/20 text-neutral-300 hover:text-emerald-300 border border-white/10 text-xs font-bold flex items-center gap-1 cursor-pointer transition-colors"
          >
            <CheckCheck class="w-3.5 h-3.5" />
            <span class="hidden md:inline">Enable All</span>
          </button>

          <button
            onclick={() => launcher.bulkToggleMods(false)}
            title="Disable all mods"
            class="p-2 rounded-xl bg-white/5 hover:bg-amber-500/20 text-neutral-300 hover:text-amber-300 border border-white/10 text-xs font-bold flex items-center gap-1 cursor-pointer transition-colors"
          >
            <XCircle class="w-3.5 h-3.5" />
            <span class="hidden md:inline">Disable All</span>
          </button>

          <button
            onclick={() => launcher.importLocalMod()}
            title="Import .jar mod file directly from disk"
            class="px-3 py-1.5 rounded-xl bg-red-600/20 hover:bg-red-600/30 border border-red-500/40 text-red-400 hover:text-red-300 text-xs font-bold flex items-center gap-1.5 cursor-pointer transition-all shrink-0"
          >
            <FileUp class="w-3.5 h-3.5" />
            <span>Import .JAR</span>
          </button>
        </div>
      </div>

      <!-- Installed Mods List -->
      {#if displayedInstalledMods.length > 0}
        <div class="space-y-2">
          {#each displayedInstalledMods as mod}
            <div class="p-3.5 rounded-xl bg-[#0e1017]/90 border border-white/8 hover:border-white/16 flex items-center justify-between gap-4 transition-all group">
              <!-- Left: Toggle & Info -->
              <div class="flex items-center gap-3.5 min-w-0">
                <button
                  onclick={() => launcher.toggleMod(mod)}
                  title={mod.is_enabled ? "Click to Disable Mod" : "Click to Enable Mod"}
                  class="transition-colors cursor-pointer text-neutral-400 hover:text-white shrink-0"
                >
                  {#if mod.is_enabled}
                    <ToggleRight class="w-6 h-6 text-emerald-400" />
                  {:else}
                    <ToggleLeft class="w-6 h-6 text-neutral-500" />
                  {/if}
                </button>

                <div class="min-w-0">
                  <div class="flex items-center gap-2">
                    <span class="text-sm font-bold truncate {mod.is_enabled ? 'text-white' : 'text-neutral-400 line-through'}">
                      {mod.name || mod.clean_name || mod.file_name}
                    </span>
                    {#if mod.version}
                      <span class="px-1.5 py-0.2 rounded bg-white/8 text-neutral-300 text-[10px] font-mono shrink-0">
                        v{mod.version}
                      </span>
                    {/if}
                  </div>

                  <div class="flex items-center gap-2 text-xs text-neutral-400 truncate font-mono mt-0.5">
                    <span class="truncate">{mod.file_name || mod.filename}</span>
                    {#if mod.author}
                      <span class="text-neutral-500">•</span>
                      <span class="text-neutral-400 font-sans truncate">by {mod.author}</span>
                    {/if}
                  </div>

                  {#if mod.description}
                    <div class="text-[11px] text-neutral-400 line-clamp-1 mt-0.5 leading-snug">
                      {mod.description}
                    </div>
                  {/if}
                </div>
              </div>

              <!-- Right: Size, Status & Actions -->
              <div class="flex items-center gap-3 shrink-0">
                <span class="text-xs font-mono text-neutral-300 hidden md:inline-block">
                  {mod.file_size_display || `${((mod.file_size_bytes || 0) / 1024).toFixed(1)} KB`}
                </span>

                <span class="text-[11px] uppercase px-2 py-0.5 rounded font-bold {mod.is_enabled ? 'bg-emerald-500/15 text-emerald-400 border border-emerald-500/25' : 'bg-neutral-500/15 text-neutral-400 border border-neutral-500/25'}">
                  {mod.is_enabled ? "Active" : "Disabled"}
                </span>

                <button
                  onclick={() => {
                    const fname = mod.file_name || mod.filename || "";
                    if (confirm(`Are you sure you want to delete "${fname}"?`)) {
                      launcher.deleteMod(fname);
                    }
                  }}
                  title="Delete mod file permanently"
                  class="p-1.5 rounded-lg text-neutral-400 hover:text-red-400 hover:bg-red-500/10 transition-colors cursor-pointer"
                >
                  <Trash2 class="w-4 h-4" />
                </button>
              </div>
            </div>
          {/each}
        </div>
      {:else if launcher.installedMods.length === 0}
        <!-- No mods installed yet -->
        <div class="p-12 text-center rounded-2xl border border-white/8 bg-white/2 space-y-4 flex flex-col items-center justify-center">
          <div class="w-14 h-14 rounded-2xl bg-red-500/10 border border-red-500/20 flex items-center justify-center text-red-400">
            <Box class="w-7 h-7" />
          </div>
          <div>
            <h3 class="text-base font-bold text-white">No Mods Installed in this Profile</h3>
            <p class="text-xs text-neutral-400 mt-1 max-w-md mx-auto">
              Your profile's mods folder is currently empty. You can browse thousands of compatible mods on the Mod Catalog or import a local `.jar` file.
            </p>
          </div>
          <div class="flex items-center gap-3 pt-2">
            <button
              onclick={() => launcher.modStudioSubTab = "catalog"}
              class="px-5 py-2.5 rounded-xl bg-red-600 hover:bg-red-500 text-white font-bold text-xs uppercase tracking-wider flex items-center gap-2 transition-all cursor-pointer shadow-lg shadow-red-600/30"
            >
              <Compass class="w-4 h-4" />
              <span>Explore Mod Catalog</span>
            </button>

            <button
              onclick={() => launcher.importLocalMod()}
              class="px-5 py-2.5 rounded-xl bg-white/6 hover:bg-white/12 border border-white/10 text-white font-bold text-xs uppercase tracking-wider flex items-center gap-2 transition-all cursor-pointer"
            >
              <FileUp class="w-4 h-4" />
              <span>Import .JAR</span>
            </button>
          </div>
        </div>
      {:else}
        <!-- Filter resulted in 0 matches -->
        <div class="p-8 text-center rounded-2xl border border-white/8 bg-white/2 space-y-2">
          <Search class="w-6 h-6 text-neutral-500 mx-auto opacity-50" />
          <p class="text-xs text-neutral-300 font-medium">No installed mods match your filter query.</p>
          <button
            onclick={() => { launcher.installedModsSearch = ""; launcher.installedModsFilter = "all"; }}
            class="text-xs text-red-400 hover:underline cursor-pointer"
          >
            Clear Search & Filters
          </button>
        </div>
      {/if}
    </div>
  {/if}

  <!-- ========================================================================= -->
  <!-- PART 2: MOD CATALOG VIEW (MODRINTH ONLINE MARKETPLACE)                    -->
  <!-- ========================================================================= -->
  {#if launcher.modStudioSubTab === "catalog"}
    <div class="space-y-5">
      <!-- 1-Click Essential FPS Suite Banner -->
      <div class="relative rounded-2xl overflow-hidden border border-red-500/30 bg-gradient-to-r from-red-950/40 via-[#100a0e] to-[#0d0f17] p-6 shadow-xl flex flex-col md:flex-row items-center justify-between gap-4">
        <div class="flex items-center gap-4">
          <div class="w-12 h-12 rounded-xl bg-red-500/20 border border-red-500/40 flex items-center justify-center text-red-400 shrink-0">
            <Sparkles class="w-6 h-6" />
          </div>
          <div>
            <div class="flex items-center gap-2">
              <h3 class="font-bold text-base text-white">DV Essential Performance Suite</h3>
              <span class="px-2 py-0.5 rounded bg-red-500/20 text-red-400 text-xs uppercase font-bold">
                1-Click Boost
              </span>
            </div>
            <p class="text-xs text-neutral-300 mt-1 leading-relaxed">
              Installs Sodium (or Embeddium on Forge), Lithium, and FerriteCore for maximum FPS rendering, smooth chunk generation, and low memory usage.
            </p>
          </div>
        </div>

        <button
          onclick={() => launcher.installEssentialFpsPack()}
          disabled={!launcher.activeInstance || launcher.installingModId !== "" || launcher.activeInstance.mod_loader.toLowerCase() === "vanilla"}
          class="w-full md:w-auto px-6 py-3 rounded-xl bg-red-600 hover:bg-red-500 text-white font-bold text-xs uppercase tracking-wider flex items-center justify-center gap-2 shadow-lg shadow-red-500/20 transition-all cursor-pointer shrink-0 disabled:opacity-50"
        >
          {#if launcher.installingModId}
            <div class="w-4 h-4 border-2 border-white/30 border-t-white rounded-full animate-spin"></div>
            <span>BOOSTING...</span>
          {:else}
            <Zap class="w-4 h-4" />
            <span>1-CLICK INSTALL ALL</span>
          {/if}
        </button>
      </div>

      <!-- Catalog Filter Bar -->
      <div class="space-y-3">
        <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-2">
          <h3 class="text-sm font-bold uppercase tracking-wider text-neutral-200 flex items-center gap-2">
            <Compass class="w-4 h-4 text-emerald-400" />
            <span>Discover Compatible Mods</span>
            {#if launcher.activeInstance}
              <span class="text-xs normal-case text-neutral-400 font-normal">
                (for <strong class="text-neutral-200">{launcher.activeInstance.mc_version}</strong> {launcher.activeInstance.mod_loader})
              </span>
            {/if}
          </h3>

          <!-- Sorting Selector -->
          <div class="flex items-center gap-2 shrink-0">
            <ArrowUpDown class="w-3.5 h-3.5 text-neutral-400" />
            <span class="text-xs text-neutral-400 font-medium">Sort by:</span>
            <select
              value={launcher.modrinthSortBy}
              onchange={handleSortChange}
              class="bg-white/5 border border-white/10 rounded-lg px-2.5 py-1 text-xs text-white outline-none cursor-pointer focus:border-red-500/50"
            >
              {#each sortOptions as opt}
                <option value={opt.id} class="bg-[#12141c] text-white">{opt.label}</option>
              {/each}
            </select>
          </div>
        </div>

        <!-- Search Input Bar -->
        <div class="flex gap-2">
          <div class="relative flex-1">
            <Search class="w-4 h-4 text-neutral-400 absolute left-3.5 top-1/2 -translate-y-1/2" />
            <input
              type="text"
              bind:value={launcher.modrinthQuery}
              onkeydown={(e) => e.key === "Enter" && handleSearch()}
              placeholder={launcher.activeInstance ? `Search ${launcher.activeInstance.mod_loader} mods for MC ${launcher.activeInstance.mc_version} (e.g. Sodium, Iris, JourneyMap, JEI)...` : "Search mods on Modrinth..."}
              class="w-full pl-10 pr-4 py-2.5 rounded-xl bg-white/5 border border-white/10 focus:border-red-500/50 focus:bg-white/8 outline-none text-xs text-white placeholder-neutral-400 transition-all"
            />
            {#if launcher.modrinthQuery}
              <button
                onclick={() => { launcher.modrinthQuery = ""; handleSearch(); }}
                class="absolute right-3 top-1/2 -translate-y-1/2 text-xs text-neutral-400 hover:text-white px-1.5 py-0.5 rounded cursor-pointer"
              >
                Clear
              </button>
            {/if}
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

        <!-- Category Filter Pills -->
        <div class="flex items-center gap-1.5 overflow-x-auto pb-1 scrollbar-none">
          <SlidersHorizontal class="w-3.5 h-3.5 text-neutral-400 shrink-0 mr-1" />
          {#each categories as cat}
            <button
              onclick={() => handleCategoryClick(cat.id)}
              class="px-3 py-1 rounded-lg text-xs font-bold transition-all cursor-pointer whitespace-nowrap {launcher.modrinthCategory === cat.id ? 'bg-red-500/20 text-red-400 border border-red-500/40 shadow-sm' : 'bg-white/4 text-neutral-400 hover:text-white hover:bg-white/8 border border-transparent'}"
            >
              {cat.label}
            </button>
          {/each}
        </div>
      </div>

      <!-- Modrinth Results Grid -->
      {#if launcher.isSearchingModrinth}
        <div class="p-12 text-center rounded-2xl border border-white/8 bg-white/2 flex flex-col items-center justify-center gap-3">
          <div class="w-6 h-6 border-2 border-red-500/30 border-t-red-500 rounded-full animate-spin"></div>
          <p class="text-xs text-neutral-300 font-medium">Querying Modrinth catalog for compatible mods...</p>
        </div>
      {:else if launcher.modrinthSearchResults.length > 0}
        <div class="grid grid-cols-1 md:grid-cols-2 gap-4 pt-1">
          {#each launcher.modrinthSearchResults as mod}
            {@const isInstalled = launcher.isModInstalled(mod)}
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
                  <div class="flex items-center justify-between gap-2">
                    <h4 class="font-bold text-sm text-white group-hover:text-red-400 transition-colors truncate">
                      {mod.title}
                    </h4>
                    <span class="text-[11px] text-neutral-400 shrink-0 truncate max-w-[100px]">
                      by {mod.author}
                    </span>
                  </div>
                  <p class="text-xs text-neutral-300 mt-1 line-clamp-2 leading-relaxed">
                    {mod.description}
                  </p>
                  <div class="flex items-center gap-3 mt-2 text-xs text-neutral-400 font-mono">
                    <span>{(mod.downloads / 1000).toFixed(0)}k downloads</span>
                    {#if mod.categories && mod.categories.length > 0}
                      <span>•</span>
                      <span class="text-neutral-300 font-sans">{mod.categories.slice(0, 2).join(", ")}</span>
                    {/if}
                  </div>
                </div>
              </div>

              <!-- Install / Installed / Uninstall Actions -->
              <div class="mt-3.5 pt-3 border-t border-white/5 flex items-center justify-between gap-2">
                {#if isInstalled}
                  <div class="flex-1 py-2 px-3 rounded-lg bg-emerald-500/15 border border-emerald-500/30 text-emerald-400 font-bold text-xs uppercase tracking-wider flex items-center justify-center gap-1.5">
                    <Check class="w-4 h-4 text-emerald-400" />
                    <span>Installed</span>
                  </div>

                  <button
                    onclick={() => launcher.uninstallCatalogMod(mod)}
                    title="Uninstall this mod"
                    class="px-3.5 py-2 rounded-lg bg-white/5 hover:bg-rose-500/20 hover:border-rose-500/30 border border-white/10 text-neutral-400 hover:text-rose-300 text-xs font-bold uppercase tracking-wider flex items-center gap-1.5 transition-colors cursor-pointer"
                  >
                    <Trash2 class="w-3.5 h-3.5" />
                    <span>Uninstall</span>
                  </button>
                {:else}
                  <button
                    onclick={() => launcher.installModrinthMod(mod.project_id)}
                    disabled={launcher.installingModId === mod.project_id || !launcher.activeInstance || launcher.activeInstance.mod_loader.toLowerCase() === 'vanilla'}
                    class="w-full py-2 rounded-lg bg-white/5 hover:bg-red-500/20 hover:border-red-500/30 border border-white/10 text-white font-bold text-xs uppercase tracking-wider flex items-center justify-center gap-2 transition-all cursor-pointer disabled:opacity-50"
                  >
                    {#if launcher.installingModId === mod.project_id}
                      <div class="w-3.5 h-3.5 border-2 border-white/30 border-t-white rounded-full animate-spin"></div>
                      <span>Downloading & Verifying...</span>
                    {:else}
                      <Download class="w-3.5 h-3.5" />
                      <span>Install Mod</span>
                    {/if}
                  </button>
                {/if}
              </div>
            </div>
          {/each}
        </div>
      {:else}
        <div class="p-12 text-center rounded-2xl border border-white/8 bg-white/2 space-y-2">
          <Box class="w-8 h-8 text-neutral-500 mx-auto opacity-50" />
          <p class="text-xs text-neutral-300 font-medium">No mods found matching your filter criteria.</p>
          <p class="text-xs text-neutral-400">Try switching the category above or searching for another term.</p>
        </div>
      {/if}
    </div>
  {/if}
</div>
