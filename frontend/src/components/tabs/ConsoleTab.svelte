<script lang="ts">
  import { launcher } from "../../state/launcher.svelte";
  import { 
    Terminal, 
    Trash2, 
    Copy, 
    ArrowDown, 
    Check
  } from "lucide-svelte";

  let consoleBox: HTMLDivElement | null = $state(null);
  let copied = $state(false);

  function clearLogs() {
    launcher.consoleLogs = [];
  }

  function copyLogs() {
    navigator.clipboard.writeText(launcher.consoleLogs.join("\n"));
    copied = true;
    setTimeout(() => { copied = false; }, 2000);
  }

  function getLogClass(line: string) {
    if (line.includes("ERROR") || line.includes("Exception") || line.includes("FATAL") || line.includes("Crash")) {
      return "text-rose-400 font-bold bg-rose-500/10 px-1.5 py-0.5 rounded";
    }
    if (line.includes("WARN")) {
      return "text-amber-300 font-medium";
    }
    if (line.includes("DEBUG")) {
      return "text-neutral-500";
    }
    return "text-neutral-200";
  }

  $effect(() => {
    // Scroll to bottom on new logs if autoScroll is enabled
    if (launcher.autoScrollLogs && consoleBox && launcher.consoleLogs.length) {
      setTimeout(() => {
        if (consoleBox) consoleBox.scrollTop = consoleBox.scrollHeight;
      }, 10);
    }
  });
</script>

<div class="h-full flex flex-col p-8 space-y-4">
  <!-- Top Bar -->
  <div class="flex items-center justify-between pb-4 border-b border-white/8 shrink-0">
    <div class="flex items-center gap-3">
      <div class="w-10 h-10 rounded-xl bg-red-500/10 border border-red-500/20 flex items-center justify-center text-red-500">
        <Terminal class="w-5 h-5" />
      </div>
      <div>
        <h2 class="text-xl font-black text-white uppercase tracking-tight flex items-center gap-2">
          <span>Live Minecraft JVM Console</span>
          {#if launcher.gameStatus === "running"}
            <span class="w-2.5 h-2.5 rounded-full bg-emerald-400 animate-pulse"></span>
          {/if}
        </h2>
        <div class="text-xs text-neutral-300 font-medium mt-0.5">
          Streaming stdout & stderr • {launcher.consoleLogs.length} events buffered
        </div>
      </div>
    </div>

    <div class="flex items-center gap-2">
      <button
        onclick={() => launcher.autoScrollLogs = !launcher.autoScrollLogs}
        class="px-4 py-2 rounded-xl border text-xs font-bold flex items-center gap-2 transition-all cursor-pointer {launcher.autoScrollLogs ? 'bg-red-500/15 border-red-500/40 text-red-400' : 'bg-white/5 border-white/10 text-neutral-300'}"
      >
        <ArrowDown class="w-4 h-4" />
        <span>Auto-Scroll</span>
      </button>

      <button
        onclick={copyLogs}
        class="px-4 py-2 rounded-xl bg-white/5 hover:bg-white/10 border border-white/10 text-neutral-200 hover:text-white text-xs font-bold flex items-center gap-2 transition-all cursor-pointer"
      >
        {#if copied}
          <Check class="w-4 h-4 text-emerald-400" />
          <span>Copied!</span>
        {:else}
          <Copy class="w-4 h-4" />
          <span>Copy</span>
        {/if}
      </button>

      <button
        onclick={clearLogs}
        class="px-4 py-2 rounded-xl bg-white/5 hover:bg-white/10 border border-white/10 text-neutral-200 hover:text-white text-xs font-bold flex items-center gap-2 transition-all cursor-pointer"
      >
        <Trash2 class="w-4 h-4" />
        <span>Clear</span>
      </button>
    </div>
  </div>

  <!-- Terminal Output Area -->
  <div 
    bind:this={consoleBox}
    class="flex-1 rounded-2xl bg-[#06070a] border border-white/8 p-5 font-mono text-xs overflow-y-auto select-text space-y-1 shadow-inner leading-relaxed"
  >
    {#if launcher.consoleLogs.length === 0}
      <div class="h-full flex flex-col items-center justify-center text-neutral-500 space-y-2 select-none">
        <Terminal class="w-8 h-8 opacity-40" />
        <span class="text-xs">No log events yet. Launch Minecraft to observe live engine outputs.</span>
      </div>
    {:else}
      {#each launcher.consoleLogs as line}
        <div class="break-all font-mono text-xs {getLogClass(line)}">
          {line}
        </div>
      {/each}
    {/if}
  </div>
</div>
