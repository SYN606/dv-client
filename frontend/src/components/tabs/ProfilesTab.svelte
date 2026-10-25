<script lang="ts">
  import { launcher } from "../../state/launcher.svelte";
  import {
    FolderKanban,
    Plus,
    FolderOpen,
    Trash2,
    Check,
    Play,
    Clock,
    Layers,
    HardDrive,
  } from "lucide-svelte";

  function getLoaderBadge(loader: string) {
    const l = loader.toLowerCase();
    if (l.includes("fabric"))
      return {
        name: "Fabric",
        color: "text-purple-300 bg-purple-500/15 border-purple-500/30",
      };
    if (l.includes("neoforge"))
      return {
        name: "NeoForge",
        color: "text-orange-300 bg-orange-500/15 border-orange-500/30",
      };
    if (l.includes("forge"))
      return {
        name: "Forge",
        color: "text-amber-300 bg-amber-500/15 border-amber-500/30",
      };
    return {
      name: "Vanilla",
      color: "text-neutral-300 bg-white/5 border-white/10",
    };
  }

  function formatLastPlayed(timestamp: number | null) {
    if (!timestamp) return "Never played";
    const date = new Date(timestamp * 1000);
    return date.toLocaleDateString(undefined, {
      month: "short",
      day: "numeric",
      hour: "2-digit",
      minute: "2-digit",
    });
  }
</script>

<div class="h-full overflow-y-auto p-4 sm:p-6 lg:p-8 space-y-5 sm:space-y-6">
  <!-- Top Bar -->
  <div class="flex items-center justify-between pb-4 border-b border-white/8">
    <div>
      <h2
        class="text-2xl font-black text-white uppercase tracking-tight flex items-center gap-3"
      >
        <FolderKanban class="w-6 h-6 text-red-500" />
        <span>Minecraft Profiles</span>
      </h2>
      <p class="text-xs text-neutral-300 mt-1 font-medium">
        Manage isolated game installations, mod loaders, and memory configs.
      </p>
    </div>

    <button
      onclick={() => (launcher.showCreateInstanceModal = true)}
      class="px-5 py-2.5 rounded-xl bg-gradient-to-r from-red-600 to-rose-600 hover:from-red-500 hover:to-rose-500 text-white font-bold text-xs uppercase tracking-wider flex items-center gap-2 shadow-lg shadow-red-500/20 transition-all cursor-pointer hover:scale-105 active:scale-95"
    >
      <Plus class="w-4 h-4" />
      <span>New Profile</span>
    </button>
  </div>

  <!-- Profiles Grid -->
  <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
    {#each launcher.instances as inst}
      {@const badge = getLoaderBadge(inst.mod_loader)}
      {@const effectiveRam = inst.is_active
        ? launcher.settings.default_max_memory_mb || inst.max_memory_mb
        : inst.max_memory_mb}
      <div
        class="rounded-2xl border p-5 flex flex-col justify-between transition-all duration-200 group relative {inst.is_active
          ? 'bg-[#140b0f] border-red-500/40 shadow-xl shadow-red-500/10'
          : 'bg-[#0e1017]/90 border-white/8 hover:border-white/20 hover:bg-[#11131c]'}"
      >
        <!-- Active Tag Indicator -->
        {#if inst.is_active}
          <div
            class="absolute -top-2.5 right-4 px-2.5 py-0.5 rounded-full bg-red-600 text-white text-[10px] font-black uppercase tracking-wider shadow-md flex items-center gap-1"
          >
            <Check class="w-3 h-3 stroke-[3]" /> Active
          </div>
        {/if}

        <div>
          <div class="flex items-start justify-between mb-3">
            <div
              class="w-12 h-12 rounded-xl bg-white/5 border border-white/10 flex items-center justify-center group-hover:scale-105 transition-transform"
            >
              <span class="font-bold text-base text-white">MC</span>
            </div>
            <span
              class="text-xs px-2.5 py-0.5 rounded-md border uppercase font-bold {badge.color}"
            >
              {badge.name}
            </span>
          </div>

          <h3
            class="text-base font-bold text-white tracking-tight group-hover:text-red-400 transition-colors truncate"
          >
            {inst.name}
          </h3>

          <div class="mt-4 space-y-2 text-xs text-neutral-300">
            <div
              class="flex items-center justify-between py-1 border-b border-white/4"
            >
              <span class="text-neutral-400 flex items-center gap-2">
                <Layers class="w-4 h-4" /> Version
              </span>
              <span class="text-white font-semibold"
                >Minecraft {inst.mc_version}</span
              >
            </div>

            <div
              class="flex items-center justify-between py-1 border-b border-white/4"
            >
              <span class="text-neutral-400 flex items-center gap-2">
                <HardDrive class="w-4 h-4 text-red-400" /> RAM Max
              </span>
              <span class="text-white font-semibold font-mono flex items-center gap-1.5">
                <span class="text-red-400 font-bold">{effectiveRam} MB</span>
                <span class="text-neutral-400 text-[11px] font-sans">({Math.round(effectiveRam / 1024)} GB)</span>
              </span>
            </div>

            <!-- Quick RAM Allocation Presets -->
            <div class="py-1.5 border-b border-white/4">
              <div class="flex items-center justify-between text-[10px] text-neutral-400 uppercase font-bold tracking-wider mb-1.5">
                <span>Allocate RAM</span>
                {#if inst.is_active}
                  <span class="text-red-400 font-mono text-[9px] lowercase">syncs with settings</span>
                {/if}
              </div>
              <div class="grid grid-cols-4 gap-1">
                {#each [2048, 4096, 6144, 8192] as ramVal}
                  <button
                    type="button"
                    onclick={(e) => {
                      e.stopPropagation();
                      launcher.updateInstanceMemory(inst.id, ramVal);
                    }}
                    class="py-1 px-1 rounded-md text-[10px] font-mono font-bold transition-all cursor-pointer border text-center {effectiveRam === ramVal
                      ? 'bg-red-600/30 text-red-200 border-red-500/60 shadow-sm'
                      : 'bg-white/4 hover:bg-white/10 text-neutral-400 hover:text-white border-white/5'}"
                  >
                    {ramVal / 1024} GB
                  </button>
                {/each}
              </div>
            </div>

            <div class="flex items-center justify-between py-1">
              <span class="text-neutral-400 flex items-center gap-2">
                <Clock class="w-4 h-4" /> Played
              </span>
              <span class="text-neutral-300"
                >{formatLastPlayed(inst.last_played)}</span
              >
            </div>
          </div>
        </div>

        <!-- Action Buttons -->
        <div
          class="mt-6 pt-4 border-t border-white/8 flex items-center justify-between gap-2"
        >
          {#if inst.is_active}
            <button
              onclick={() => {
                launcher.activeTab = "play";
                launcher.launchOrStop();
              }}
              class="flex-1 py-2.5 rounded-xl bg-red-600 hover:bg-red-500 text-white text-xs font-bold uppercase tracking-wider flex items-center justify-center gap-2 transition-all cursor-pointer shadow-md"
            >
              <Play class="w-3.5 h-3.5 fill-white" />
              <span>Launch</span>
            </button>
          {:else}
            <button
              onclick={() => launcher.selectInstance(inst.id)}
              class="flex-1 py-2.5 rounded-xl bg-white/5 hover:bg-white/10 text-white border border-white/10 text-xs font-bold uppercase tracking-wider flex items-center justify-center gap-1.5 transition-all cursor-pointer"
            >
              <span>Select Profile</span>
            </button>
          {/if}

          <button
            onclick={() => launcher.openModsFolder()}
            title="Open Mods Folder"
            class="p-2.5 rounded-xl bg-white/5 hover:bg-white/10 text-neutral-300 hover:text-white border border-white/10 transition-all cursor-pointer"
          >
            <FolderOpen class="w-4 h-4" />
          </button>

          <button
            onclick={() => launcher.deleteInstance(inst.id)}
            title="Delete Profile"
            class="p-2.5 rounded-xl bg-white/5 hover:bg-red-500/20 text-neutral-300 hover:text-red-400 border border-white/10 transition-all cursor-pointer"
          >
            <Trash2 class="w-4 h-4" />
          </button>
        </div>
      </div>
    {/each}

    {#if launcher.instances.length === 0}
      <div
        class="col-span-full py-16 text-center space-y-4 rounded-3xl border border-dashed border-white/10 bg-white/2"
      >
        <div
          class="w-16 h-16 rounded-2xl bg-red-500/10 border border-red-500/20 mx-auto flex items-center justify-center text-red-500"
        >
          <FolderKanban class="w-8 h-8" />
        </div>
        <div class="space-y-1">
          <h3 class="text-base font-bold text-white">
            No Minecraft Profiles Created Yet
          </h3>
          <p class="text-xs text-neutral-300">
            Create your first profile to download Minecraft assets, install
            mods, and play.
          </p>
        </div>
        <button
          onclick={() => (launcher.showCreateInstanceModal = true)}
          class="px-6 py-2.5 rounded-xl bg-red-600 hover:bg-red-500 text-white font-bold text-xs uppercase tracking-wider inline-flex items-center gap-2 shadow-lg shadow-red-500/25 transition-all cursor-pointer"
        >
          <Plus class="w-4 h-4" />
          <span>Create First Profile</span>
        </button>
      </div>
    {/if}
  </div>
</div>
