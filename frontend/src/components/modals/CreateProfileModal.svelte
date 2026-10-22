<script lang="ts">
  import { launcher } from "../../state/launcher.svelte";
  import { 
    X, 
    FolderPlus, 
    Sparkles, 
    Check, 
    Layers, 
    Box 
  } from "lucide-svelte";

  let profileName = $state("1.21.4 - Fast Performance");
  let mcVersion = $state("1.21.4");
  let modLoader = $state("Fabric");

  const popularVersions = [
    { ver: "1.21.4", label: "1.21.4 (Latest)", desc: "Best Performance & Tricky Trials" },
    { ver: "1.20.1", label: "1.20.1", desc: "Huge Modpack Ecosystem" },
    { ver: "1.16.5", label: "1.16.5", desc: "Nether Update Era" },
    { ver: "1.12.2", label: "1.12.2", desc: "Classic Forge Mods" },
    { ver: "1.8.9", label: "1.8.9", desc: "Hypixel & Competitive PvP" },
  ];

  const loaders = [
    { id: "Fabric", label: "Fabric", desc: "Fastest engine, Sodium compatible", color: "text-purple-400 border-purple-500/30" },
    { id: "Vanilla", label: "Vanilla", desc: "Pure official unmodded Minecraft", color: "text-neutral-300 border-white/20" },
    { id: "NeoForge", label: "NeoForge", desc: "Modern Forge fork for 1.20.2+", color: "text-orange-400 border-orange-500/30" },
    { id: "Forge", label: "Forge", desc: "Traditional Minecraft forge loader", color: "text-amber-400 border-amber-500/30" },
  ];

  function handleCreate() {
    if (!profileName.trim()) return;
    launcher.createInstance(profileName.trim(), mcVersion, modLoader);
  }
</script>

{#if launcher.showCreateInstanceModal}
  <div class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/80 backdrop-blur-md animate-in fade-in duration-200">
    <div class="relative w-full max-w-lg rounded-3xl bg-[#0e1017] border border-white/10 shadow-2xl p-6 space-y-6">
      <!-- Header -->
      <div class="flex items-center justify-between pb-4 border-b border-white/8">
        <div class="flex items-center gap-3">
          <div class="w-10 h-10 rounded-xl bg-red-500/10 border border-red-500/20 flex items-center justify-center text-red-500">
            <FolderPlus class="w-5 h-5" />
          </div>
          <div>
            <h3 class="text-base font-black uppercase text-white font-mono">Create Minecraft Profile</h3>
            <p class="text-xs text-neutral-400 font-mono">Configure version and performance mod loader</p>
          </div>
        </div>

        <button
          onclick={() => launcher.showCreateInstanceModal = false}
          class="p-2 rounded-xl bg-white/5 hover:bg-white/10 text-neutral-400 hover:text-white transition-all cursor-pointer"
        >
          <X class="w-4 h-4" />
        </button>
      </div>

      <!-- Form Inputs -->
      <div class="space-y-4">
        <!-- Name -->
        <div>
          <div class="text-xs font-mono font-bold uppercase text-neutral-300 mb-1.5">Profile Name</div>
          <input
            type="text"
            bind:value={profileName}
            placeholder="e.g. 1.21.4 Survival, PvP Client..."
            class="w-full px-4 py-2.5 rounded-xl bg-white/4 border border-white/8 text-xs font-mono text-white outline-none focus:border-red-500/50"
          />
        </div>

        <!-- MC Version Selection -->
        <div>
          <div class="text-xs font-mono font-bold uppercase text-neutral-300 mb-1.5">Minecraft Version</div>
          <div class="grid grid-cols-2 gap-2">
            {#each popularVersions as v}
              <button
                onclick={() => {
                  mcVersion = v.ver;
                  profileName = `${v.ver} - ${modLoader}`;
                }}
                class="p-2.5 rounded-xl border text-left transition-all cursor-pointer {mcVersion === v.ver ? 'bg-red-500/15 border-red-500/40 text-white' : 'bg-white/3 border-white/6 text-neutral-400 hover:text-white'}"
              >
                <div class="text-xs font-bold font-mono">{v.label}</div>
                <div class="text-[10px] font-mono text-neutral-500 truncate">{v.desc}</div>
              </button>
            {/each}
          </div>
        </div>

        <!-- Mod Loader Selection -->
        <div>
          <div class="text-xs font-mono font-bold uppercase text-neutral-300 mb-1.5">Mod Loader</div>
          <div class="grid grid-cols-2 gap-2">
            {#each loaders as l}
              <button
                onclick={() => {
                  modLoader = l.id;
                  profileName = `${mcVersion} - ${l.id}`;
                }}
                class="p-2.5 rounded-xl border text-left transition-all cursor-pointer {modLoader === l.id ? 'bg-red-500/15 border-red-500/40 text-white' : 'bg-white/3 border-white/6 text-neutral-400 hover:text-white'}"
              >
                <div class="text-xs font-bold font-mono">{l.label}</div>
                <div class="text-[10px] font-mono text-neutral-500 truncate">{l.desc}</div>
              </button>
            {/each}
          </div>
        </div>
      </div>

      <!-- Action Buttons -->
      <div class="flex items-center justify-end gap-3 pt-4 border-t border-white/8">
        <button
          onclick={() => launcher.showCreateInstanceModal = false}
          class="px-5 py-2.5 rounded-xl bg-white/5 hover:bg-white/10 text-neutral-300 text-xs font-mono font-bold uppercase transition-all cursor-pointer"
        >
          Cancel
        </button>

        <button
          onclick={handleCreate}
          class="px-6 py-2.5 rounded-xl bg-gradient-to-r from-red-600 to-rose-600 hover:from-red-500 hover:to-rose-500 text-white text-xs font-mono font-bold uppercase tracking-wider flex items-center gap-2 shadow-lg shadow-red-500/25 transition-all cursor-pointer"
        >
          <Check class="w-4 h-4" />
          <span>Create Profile</span>
        </button>
      </div>
    </div>
  </div>
{/if}
