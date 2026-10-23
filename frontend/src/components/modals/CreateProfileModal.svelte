<script lang="ts">
  import { onMount } from "svelte";
  import { launcher } from "../../state/launcher.svelte";
  import {
    POPULAR_MINECRAFT_VERSIONS,
    SUPPORTED_MOD_LOADERS,
  } from "../../constants/minecraft";
  import { formatISODate } from "../../utils/formatters";
  import {
    X,
    FolderPlus,
    Check,
    Search,
    Calendar,
    Sparkles,
    Filter,
    ChevronDown,
    ChevronUp,
    Layers,
    Clock,
  } from "lucide-svelte";

  let profileName = $state("1.21.4 - Fabric");
  let mcVersion = $state("1.21.4");
  let modLoader = $state("Fabric");

  // Version browser states
  let showAllVersions = $state(false);
  let versionSearch = $state("");
  let versionCategory = $state<"release" | "snapshot" | "historical" | "all">(
    "release",
  );

  const popularVersions = POPULAR_MINECRAFT_VERSIONS;
  const loaders = SUPPORTED_MOD_LOADERS;

  onMount(() => {
    if (launcher.availableMinecraftVersions.length === 0) {
      launcher.loadMinecraftVersions();
    }
  });

  $effect(() => {
    if (
      launcher.showCreateInstanceModal &&
      launcher.availableMinecraftVersions.length === 0
    ) {
      launcher.loadMinecraftVersions();
    }
  });

  let filteredVersions = $derived.by(() => {
    let list = launcher.availableMinecraftVersions;
    if (!list || list.length === 0) return [];

    // Filter by category
    if (versionCategory === "release") {
      list = list.filter((v) => v.version_type === "release");
    } else if (versionCategory === "snapshot") {
      list = list.filter((v) => v.version_type === "snapshot");
    } else if (versionCategory === "historical") {
      list = list.filter(
        (v) =>
          v.version_type === "old_beta" ||
          v.version_type === "old_alpha" ||
          v.version_type.includes("old"),
      );
    }

    // Filter by search query
    if (versionSearch.trim()) {
      const q = versionSearch.trim().toLowerCase();
      list = list.filter((v) => v.id.toLowerCase().includes(q));
    }

    return list;
  });

  function selectVersion(vId: string) {
    mcVersion = vId;
    profileName = `${vId} - ${modLoader}`;
  }

  function selectLoader(lId: string) {
    modLoader = lId;
    profileName = `${mcVersion} - ${lId}`;
  }

  function handleCreate() {
    if (!profileName.trim()) return;
    launcher.createInstance(profileName.trim(), mcVersion, modLoader);
  }
</script>

{#if launcher.showCreateInstanceModal}
  <div
    class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/80 backdrop-blur-md animate-in fade-in duration-200"
  >
    <div
      class="relative w-full max-w-xl max-h-[92vh] flex flex-col rounded-3xl bg-[#0e1017] border border-white/10 shadow-2xl overflow-hidden"
    >
      <!-- Header -->
      <div
        class="flex items-center justify-between p-6 pb-4 border-b border-white/8 shrink-0"
      >
        <div class="flex items-center gap-3">
          <div
            class="w-10 h-10 rounded-xl bg-red-500/10 border border-red-500/20 flex items-center justify-center text-red-500 shrink-0"
          >
            <FolderPlus class="w-5 h-5" />
          </div>
          <div>
            <h3 class="text-base font-bold uppercase text-white tracking-wide">
              Create Minecraft Profile
            </h3>
            <p class="text-xs text-neutral-300">
              Configure profile name, Minecraft version (all 600+ builds), and
              loader
            </p>
          </div>
        </div>

        <button
          onclick={() => (launcher.showCreateInstanceModal = false)}
          class="p-2 rounded-xl bg-white/5 hover:bg-white/10 text-neutral-300 hover:text-white transition-all cursor-pointer"
        >
          <X class="w-4 h-4" />
        </button>
      </div>

      <!-- Scrollable Form Body -->
      <div class="p-6 space-y-5 overflow-y-auto flex-1 scrollbar-thin">
        <!-- Profile Name -->
        <div>
          <label
            for="profile-name-input"
            class="block text-xs font-bold uppercase tracking-wider text-neutral-200 mb-1.5"
          >
            Profile Name
          </label>
          <input
            id="profile-name-input"
            type="text"
            bind:value={profileName}
            placeholder="e.g. 1.21.4 Survival, PvP Client..."
            class="w-full px-4 py-2.5 rounded-xl bg-white/5 border border-white/10 text-xs text-white outline-none focus:border-red-500/50 transition-colors"
          />
        </div>

        <!-- Minecraft Version Section -->
        <div>
          <div class="flex items-center justify-between mb-2">
            <span
              class="text-xs font-bold uppercase tracking-wider text-neutral-200 flex items-center gap-2"
            >
              <span>Minecraft Version:</span>
              <span
                class="px-2 py-0.5 rounded-md bg-red-500/20 text-red-400 border border-red-500/30 text-xs font-mono font-bold"
              >
                {mcVersion}
              </span>
            </span>

            <button
              type="button"
              onclick={() => (showAllVersions = !showAllVersions)}
              class="text-xs text-neutral-400 hover:text-white flex items-center gap-1 cursor-pointer transition-colors"
            >
              <span>{showAllVersions ? "Show Popular" : "Browse All 600+ Versions"}</span>
              {#if showAllVersions}
                <ChevronUp class="w-3.5 h-3.5" />
              {:else}
                <ChevronDown class="w-3.5 h-3.5" />
              {/if}
            </button>
          </div>

          <!-- Mode 1: Quick Popular Version Chips -->
          {#if !showAllVersions}
            <div class="grid grid-cols-2 sm:grid-cols-3 gap-2">
              {#each popularVersions as v}
                <button
                  type="button"
                  onclick={() => selectVersion(v.ver)}
                  class="p-2.5 rounded-xl border text-left transition-all cursor-pointer relative {mcVersion ===
                  v.ver
                    ? 'bg-red-500/15 border-red-500/40 text-white shadow-sm shadow-red-500/10'
                    : 'bg-white/4 border-white/8 text-neutral-300 hover:text-white hover:bg-white/6'}"
                >
                  <div class="flex items-center justify-between gap-1">
                    <span class="text-xs font-bold font-mono">{v.label}</span>
                    <span
                      class="px-1.5 py-0.2 rounded text-[10px] font-bold uppercase {v.tag ===
                      'Latest'
                        ? 'bg-emerald-500/20 text-emerald-300 border border-emerald-500/30'
                        : 'bg-white/10 text-neutral-400'}"
                    >
                      {v.tag}
                    </span>
                  </div>
                  <div class="text-[11px] text-neutral-400 mt-0.5 truncate">
                    {v.desc}
                  </div>
                </button>
              {/each}
            </div>
          {/if}

          <!-- Mode 2: Full Version Browser (Official Mojang Manifest) -->
          {#if showAllVersions}
            <div
              class="rounded-2xl bg-black/40 border border-white/10 p-3 space-y-3"
            >
              <!-- Category Filter Pills & Search Input -->
              <div
                class="flex flex-col sm:flex-row items-stretch sm:items-center justify-between gap-2"
              >
                <!-- Category Pills -->
                <div class="flex items-center gap-1 bg-white/4 p-1 rounded-xl">
                  <button
                    type="button"
                    onclick={() => (versionCategory = "release")}
                    class="px-2.5 py-1 rounded-lg text-xs font-bold transition-all cursor-pointer {versionCategory ===
                    'release'
                      ? 'bg-red-600 text-white shadow-sm'
                      : 'text-neutral-400 hover:text-white'}"
                  >
                    Releases
                  </button>
                  <button
                    type="button"
                    onclick={() => (versionCategory = "snapshot")}
                    class="px-2.5 py-1 rounded-lg text-xs font-bold transition-all cursor-pointer {versionCategory ===
                    'snapshot'
                      ? 'bg-red-600 text-white shadow-sm'
                      : 'text-neutral-400 hover:text-white'}"
                  >
                    Snapshots
                  </button>
                  <button
                    type="button"
                    onclick={() => (versionCategory = "historical")}
                    class="px-2.5 py-1 rounded-lg text-xs font-bold transition-all cursor-pointer {versionCategory ===
                    'historical'
                      ? 'bg-red-600 text-white shadow-sm'
                      : 'text-neutral-400 hover:text-white'}"
                  >
                    Beta/Alpha
                  </button>
                  <button
                    type="button"
                    onclick={() => (versionCategory = "all")}
                    class="px-2.5 py-1 rounded-lg text-xs font-bold transition-all cursor-pointer {versionCategory ===
                    'all'
                      ? 'bg-red-600 text-white shadow-sm'
                      : 'text-neutral-400 hover:text-white'}"
                  >
                    All
                  </button>
                </div>

                <!-- Live Search Bar -->
                <div class="relative flex-1">
                  <Search
                    class="w-3.5 h-3.5 text-neutral-400 absolute left-2.5 top-1/2 -translate-y-1/2"
                  />
                  <input
                    type="text"
                    bind:value={versionSearch}
                    placeholder="Search version (e.g. 1.20, 24w, b1.7)..."
                    class="w-full pl-8 pr-3 py-1.5 rounded-xl bg-white/5 border border-white/10 text-xs text-white placeholder-neutral-400 outline-none focus:border-red-500/50"
                  />
                </div>
              </div>

              <!-- Versions Scrollable List -->
              <div
                class="max-h-48 overflow-y-auto divide-y divide-white/5 border border-white/8 rounded-xl bg-[#090b10] scrollbar-thin"
              >
                {#if launcher.isLoadingVersions && launcher.availableMinecraftVersions.length === 0}
                  <div class="p-6 text-center text-xs text-neutral-400 flex items-center justify-center gap-2">
                    <div class="w-3.5 h-3.5 border-2 border-red-500/30 border-t-red-500 rounded-full animate-spin"></div>
                    <span>Loading official Mojang version manifest...</span>
                  </div>
                {:else if filteredVersions.length > 0}
                  {#each filteredVersions as v}
                    {@const isSelected = mcVersion === v.id}
                    <button
                      type="button"
                      onclick={() => selectVersion(v.id)}
                      class="w-full px-3.5 py-2 flex items-center justify-between text-left transition-colors cursor-pointer {isSelected
                        ? 'bg-red-500/20 text-white'
                        : 'hover:bg-white/4 text-neutral-300 hover:text-white'}"
                    >
                      <div class="flex items-center gap-2.5 min-w-0">
                        <span class="text-xs font-mono font-bold">{v.id}</span>
                        {#if v.is_latest_release}
                          <span
                            class="px-1.5 py-0.2 rounded text-[10px] font-bold uppercase bg-emerald-500/20 text-emerald-300 border border-emerald-500/30"
                          >
                            Latest Release
                          </span>
                        {:else if v.is_latest_snapshot}
                          <span
                            class="px-1.5 py-0.2 rounded text-[10px] font-bold uppercase bg-purple-500/20 text-purple-300 border border-purple-500/30"
                          >
                            Latest Snapshot
                          </span>
                        {:else}
                          <span
                            class="px-1.5 py-0.2 rounded text-[10px] font-mono uppercase bg-white/6 text-neutral-400"
                          >
                            {v.version_type}
                          </span>
                        {/if}
                      </div>

                      <div class="flex items-center gap-2 text-xs text-neutral-400 font-mono shrink-0">
                        {#if v.release_time}
                          <span>{formatISODate(v.release_time)}</span>
                        {/if}
                        {#if isSelected}
                          <Check class="w-3.5 h-3.5 text-red-400" />
                        {/if}
                      </div>
                    </button>
                  {/each}
                {:else}
                  <div class="p-6 text-center text-xs text-neutral-400">
                    No versions found matching "{versionSearch}".
                  </div>
                {/if}
              </div>

              <div class="text-[11px] text-neutral-400 flex items-center justify-between px-1">
                <span>Showing {filteredVersions.length} of {launcher.availableMinecraftVersions.length} versions</span>
                <span class="text-neutral-400">Synced with Mojang Piston Manifest</span>
              </div>
            </div>
          {/if}
        </div>

        <!-- Mod Loader Selection -->
        <div>
          <div
            class="text-xs font-bold uppercase tracking-wider text-neutral-200 mb-1.5"
          >
            Mod Loader
          </div>
          <div class="grid grid-cols-2 gap-2.5">
            {#each loaders as l}
              <button
                type="button"
                onclick={() => selectLoader(l.id)}
                class="p-3 rounded-xl border text-left transition-all cursor-pointer {modLoader ===
                l.id
                  ? 'bg-red-500/15 border-red-500/40 text-white shadow-sm shadow-red-500/10'
                  : 'bg-white/4 border-white/8 text-neutral-300 hover:text-white hover:bg-white/6'}"
              >
                <div class="flex items-center justify-between">
                  <span class="text-xs font-bold">{l.label}</span>
                  {#if modLoader === l.id}
                    <Check class="w-3.5 h-3.5 text-red-400" />
                  {/if}
                </div>
                <div class="text-xs text-neutral-400 mt-0.5 truncate">
                  {l.desc}
                </div>
              </button>
            {/each}
          </div>
        </div>
      </div>

      <!-- Action Buttons Footer -->
      <div
        class="flex items-center justify-end gap-3 p-6 pt-4 border-t border-white/8 bg-[#090b10]/60 shrink-0"
      >
        <button
          type="button"
          onclick={() => (launcher.showCreateInstanceModal = false)}
          class="px-5 py-2.5 rounded-xl bg-white/5 hover:bg-white/10 text-neutral-300 text-xs font-bold uppercase transition-all cursor-pointer"
        >
          Cancel
        </button>

        <button
          type="button"
          onclick={handleCreate}
          class="px-6 py-2.5 rounded-xl bg-gradient-to-r from-red-600 to-rose-600 hover:from-red-500 hover:to-rose-500 text-white text-xs font-bold uppercase tracking-wider flex items-center gap-2 shadow-lg shadow-red-500/25 transition-all cursor-pointer"
        >
          <Check class="w-4 h-4" />
          <span>Create Profile</span>
        </button>
      </div>
    </div>
  </div>
{/if}
