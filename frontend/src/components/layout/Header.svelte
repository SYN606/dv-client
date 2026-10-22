<script lang="ts">
  import { launcher } from "../../state/launcher.svelte";
  import { 
    ChevronDown, 
    User, 
    ShieldCheck, 
    HardDrive, 
    Plus, 
    Zap,
    Box
  } from "lucide-svelte";

  let showInstanceDropdown = $state(false);

  function getLoaderColor(loader: string) {
    const l = loader.toLowerCase();
    if (l.includes("fabric")) return "text-purple-300 bg-purple-500/15 border-purple-500/30";
    if (l.includes("neoforge")) return "text-orange-300 bg-orange-500/15 border-orange-500/30";
    if (l.includes("forge")) return "text-amber-300 bg-amber-500/15 border-amber-500/30";
    return "text-neutral-300 bg-neutral-500/15 border-neutral-500/30";
  }

  function getAvatarUrl() {
    if (!launcher.activeAccount) return null;
    if (launcher.activeAccount.avatar_url) return launcher.activeAccount.avatar_url;
    if (launcher.activeAccount.account_type === "microsoft" && launcher.activeAccount.id) {
      return `https://crafthead.net/avatar/${launcher.activeAccount.id}`;
    }
    return `https://crafthead.net/avatar/${launcher.activeAccount.username}`;
  }
</script>

<header class="h-16 border-b border-white/8 bg-[#0a0c12]/90 backdrop-blur-xl px-6 flex items-center justify-between shrink-0 z-30 select-none">
  <!-- Left: Active Profile Quick Selector -->
  <div class="relative">
    <button
      onclick={() => showInstanceDropdown = !showInstanceDropdown}
      class="flex items-center gap-3 px-3.5 py-2 rounded-xl bg-white/5 hover:bg-white/10 border border-white/10 transition-all cursor-pointer group"
    >
      <div class="w-8 h-8 rounded-lg bg-red-500/15 border border-red-500/30 flex items-center justify-center text-red-400 group-hover:scale-105 transition-transform">
        <Box class="w-4 h-4" />
      </div>

      <div class="text-left">
        <div class="flex items-center gap-2">
          <span class="text-sm font-bold text-white tracking-tight">
            {launcher.activeInstance?.name || "No Profile Selected"}
          </span>
          {#if launcher.activeInstance}
            <span class="text-[11px] px-2 py-0.5 rounded border {getLoaderColor(launcher.activeInstance.mod_loader)} uppercase font-semibold">
              {launcher.activeInstance.mod_loader}
            </span>
          {/if}
        </div>
        <div class="text-xs text-neutral-300 font-medium">
          {launcher.activeInstance ? `Minecraft ${launcher.activeInstance.mc_version}` : "Select a profile to play"}
        </div>
      </div>

      <ChevronDown class="w-4 h-4 text-neutral-400 group-hover:text-white transition-transform {showInstanceDropdown ? 'rotate-180' : ''}" />
    </button>

    <!-- Dropdown Menu -->
    {#if showInstanceDropdown}
      <div 
        class="absolute left-0 mt-2 w-76 rounded-2xl bg-[#0f121b] border border-white/12 shadow-2xl p-2.5 z-50 backdrop-blur-3xl animate-in fade-in slide-in-from-top-2 duration-150"
      >
        <div class="px-3 py-2 text-xs font-bold uppercase tracking-wider text-neutral-400 flex items-center justify-between border-b border-white/8 mb-1.5">
          <span>Switch Profile</span>
          <button 
            onclick={() => { showInstanceDropdown = false; launcher.showCreateInstanceModal = true; }}
            class="text-red-400 hover:text-red-300 flex items-center gap-1 cursor-pointer font-semibold"
          >
            <Plus class="w-3.5 h-3.5" /> New Profile
          </button>
        </div>

        <div class="max-h-64 overflow-y-auto space-y-1">
          {#each launcher.instances as inst}
            <button
              onclick={() => {
                launcher.selectInstance(inst.id);
                showInstanceDropdown = false;
              }}
              class="w-full flex items-center justify-between p-2.5 rounded-xl text-left transition-all cursor-pointer {inst.is_active ? 'bg-red-500/15 border border-red-500/30 text-white' : 'hover:bg-white/5 text-neutral-200'}"
            >
              <div class="truncate">
                <div class="text-sm font-semibold truncate">{inst.name}</div>
                <div class="text-xs text-neutral-400">MC {inst.mc_version} • {inst.mod_loader}</div>
              </div>
              {#if inst.is_active}
                <span class="w-2.5 h-2.5 rounded-full bg-red-400 shrink-0"></span>
              {/if}
            </button>
          {/each}

          {#if launcher.instances.length === 0}
            <div class="p-4 text-center text-xs text-neutral-400">
              No profiles found. Create your first profile!
            </div>
          {/if}
        </div>
      </div>
    {/if}
  </div>

  <!-- Center: Performance Engine Indicator -->
  <div class="hidden md:flex items-center gap-3 px-4 py-2 rounded-full bg-white/4 border border-white/8 text-xs text-neutral-300">
    <div class="flex items-center gap-1.5 text-red-400 font-bold uppercase tracking-wide">
      <Zap class="w-4 h-4 fill-red-400/20" />
      <span>DV TURBO ENGINE</span>
    </div>
    <span class="text-neutral-600">•</span>
    <div class="flex items-center gap-1.5 text-neutral-200 font-medium">
      <HardDrive class="w-3.5 h-3.5 text-neutral-400" />
      <span class="font-mono">{launcher.settings.default_max_memory_mb || 4096} MB RAM</span>
    </div>
  </div>

  <!-- Right: Account Profile Pill -->
  <div class="flex items-center gap-3">
    <button
      onclick={() => launcher.showAccountModal = true}
      class="flex items-center gap-3 px-3.5 py-2 rounded-xl bg-white/5 hover:bg-white/10 border border-white/10 transition-all cursor-pointer group"
    >
      {#if launcher.activeAccount}
        <div class="relative w-8 h-8 rounded-lg overflow-hidden bg-black/40 border border-white/10 shrink-0">
          <img 
            src={getAvatarUrl()} 
            alt={launcher.activeAccount.username}
            class="w-full h-full object-cover rendering-pixelated"
            onerror={(e) => {
              (e.currentTarget as HTMLElement).style.display = 'none';
            }}
          />
        </div>
        <div class="text-left">
          <div class="flex items-center gap-1.5">
            <span class="text-sm font-bold text-white tracking-tight">{launcher.activeAccount.username}</span>
            {#if launcher.activeAccount.account_type === "microsoft"}
              <ShieldCheck class="w-3.5 h-3.5 text-sky-400" />
            {/if}
          </div>
          <div class="text-xs text-neutral-400 font-medium">
            {launcher.activeAccount.account_type === "microsoft" ? "Microsoft Account" : "Offline Player"}
          </div>
        </div>
      {:else}
        <div class="w-8 h-8 rounded-lg bg-red-500/10 border border-red-500/20 flex items-center justify-center text-red-400">
          <User class="w-4 h-4" />
        </div>
        <div class="text-left">
          <div class="text-sm font-bold text-neutral-200">No Account</div>
          <div class="text-xs text-neutral-400">Click to connect</div>
        </div>
      {/if}
      <ChevronDown class="w-4 h-4 text-neutral-400 group-hover:text-white transition-colors" />
    </button>
  </div>
</header>
