<script lang="ts">
  import { launcher } from "../../state/launcher.svelte";
  import {
    Terminal,
    Trash2,
    Copy,
    ArrowDown,
    Check,
    AlertTriangle,
    Search,
    FolderOpen,
    FileDown,
    FileText,
    X,
    Filter,
  } from "lucide-svelte";

  let consoleBox: HTMLDivElement | null = $state(null);
  let copied = $state(false);
  let crashCopied = $state(false);
  let searchQuery = $state("");
  let selectedLevel = $state<"ALL" | "INFO" | "WARN" | "ERROR">("ALL");

  function clearLogs() {
    launcher.consoleLogs = [];
  }

  function copyLogs() {
    navigator.clipboard.writeText(launcher.consoleLogs.join("\n"));
    copied = true;
    setTimeout(() => {
      copied = false;
    }, 2000);
  }

  function copyCrashSnippet() {
    navigator.clipboard.writeText(launcher.crashReportSnippet);
    crashCopied = true;
    setTimeout(() => {
      crashCopied = false;
    }, 2500);
  }

  let errorCount = $derived(
    launcher.consoleLogs.filter(
      (l) =>
        l.includes("ERROR") ||
        l.includes("FATAL") ||
        l.includes("Exception") ||
        l.includes("Crash"),
    ).length,
  );

  let warnCount = $derived(
    launcher.consoleLogs.filter((l) => l.includes("WARN")).length,
  );

  let infoCount = $derived(
    launcher.consoleLogs.filter((l) => l.includes("INFO")).length,
  );

  let displayedLogs = $derived.by(() => {
    let list = launcher.consoleLogs;
    if (selectedLevel === "INFO") {
      list = list.filter((l) => l.includes("INFO"));
    } else if (selectedLevel === "WARN") {
      list = list.filter((l) => l.includes("WARN"));
    } else if (selectedLevel === "ERROR") {
      list = list.filter(
        (l) =>
          l.includes("ERROR") ||
          l.includes("FATAL") ||
          l.includes("Exception") ||
          l.includes("Crash"),
      );
    }

    if (searchQuery.trim()) {
      const q = searchQuery.toLowerCase();
      list = list.filter((l) => l.toLowerCase().includes(q));
    }
    return list;
  });

  function getLogClass(line: string) {
    if (
      line.includes("ERROR") ||
      line.includes("Exception") ||
      line.includes("FATAL") ||
      line.includes("Crash") ||
      line.includes("Caused by:")
    ) {
      return "text-rose-400 font-semibold bg-rose-500/10 px-1.5 py-0.5 rounded border border-rose-500/15";
    }
    if (line.includes("WARN")) {
      return "text-amber-300 font-medium";
    }
    if (line.includes("DEBUG") || line.includes("TRACE")) {
      return "text-neutral-500";
    }
    return "text-neutral-200";
  }

  $effect(() => {
    // Scroll to bottom on new logs if autoScroll is enabled
    if (launcher.autoScrollLogs && consoleBox && displayedLogs.length) {
      setTimeout(() => {
        if (consoleBox) consoleBox.scrollTop = consoleBox.scrollHeight;
      }, 10);
    }
  });
</script>

<div class="h-full flex flex-col p-8 space-y-4">
  <!-- Top Bar -->
  <div
    class="flex flex-col md:flex-row md:items-center justify-between gap-4 pb-4 border-b border-white/8 shrink-0"
  >
    <div class="flex items-center gap-3">
      <div
        class="w-10 h-10 rounded-xl bg-red-500/10 border border-red-500/20 flex items-center justify-center text-red-500 shrink-0"
      >
        <Terminal class="w-5 h-5" />
      </div>
      <div>
        <h2
          class="text-xl font-black text-white uppercase tracking-tight flex items-center gap-2"
        >
          <span>Live Minecraft JVM Console</span>
          {#if launcher.gameStatus === "running"}
            <span
              class="w-2.5 h-2.5 rounded-full bg-emerald-400 animate-pulse"
              title="Game is actively running"
            ></span>
          {/if}
        </h2>
        <div class="text-xs text-neutral-300 font-medium mt-0.5">
          Streaming stdout & stderr • <strong class="text-white"
            >{launcher.consoleLogs.length}</strong
          >
          events buffered
          {#if displayedLogs.length !== launcher.consoleLogs.length}
            • <span class="text-red-400 font-bold"
              >{displayedLogs.length} matching filter</span
            >
          {/if}
        </div>
      </div>
    </div>

    <!-- Actions -->
    <div class="flex flex-wrap items-center gap-2">
      <button
        onclick={() => launcher.loadLatestLogs()}
        title="Load latest.log from game disk directory"
        class="px-3 py-1.5 rounded-xl bg-white/5 hover:bg-white/10 border border-white/10 text-neutral-200 hover:text-white text-xs font-bold flex items-center gap-1.5 transition-all cursor-pointer"
      >
        <FileText class="w-3.5 h-3.5" />
        <span class="hidden sm:inline">Load latest.log</span>
      </button>

      <button
        onclick={() => launcher.openLogsFolder()}
        title="Open logs folder on disk"
        class="px-3 py-1.5 rounded-xl bg-white/5 hover:bg-white/10 border border-white/10 text-neutral-200 hover:text-white text-xs font-bold flex items-center gap-1.5 transition-all cursor-pointer"
      >
        <FolderOpen class="w-3.5 h-3.5" />
        <span class="hidden sm:inline">Logs Folder</span>
      </button>

      <button
        onclick={() => launcher.exportLogs()}
        title="Export console logs to a .log file"
        class="px-3 py-1.5 rounded-xl bg-white/5 hover:bg-white/10 border border-white/10 text-neutral-200 hover:text-white text-xs font-bold flex items-center gap-1.5 transition-all cursor-pointer"
      >
        <FileDown class="w-3.5 h-3.5" />
        <span class="hidden sm:inline">Export</span>
      </button>

      <button
        onclick={() => (launcher.autoScrollLogs = !launcher.autoScrollLogs)}
        class="px-3 py-1.5 rounded-xl border text-xs font-bold flex items-center gap-1.5 transition-all cursor-pointer {launcher.autoScrollLogs
          ? 'bg-red-500/15 border-red-500/40 text-red-400'
          : 'bg-white/5 border-white/10 text-neutral-300'}"
      >
        <ArrowDown class="w-3.5 h-3.5" />
        <span>Auto-Scroll</span>
      </button>

      <button
        onclick={copyLogs}
        class="px-3 py-1.5 rounded-xl bg-white/5 hover:bg-white/10 border border-white/10 text-neutral-200 hover:text-white text-xs font-bold flex items-center gap-1.5 transition-all cursor-pointer"
      >
        {#if copied}
          <Check class="w-3.5 h-3.5 text-emerald-400" />
          <span>Copied!</span>
        {:else}
          <Copy class="w-3.5 h-3.5" />
          <span>Copy</span>
        {/if}
      </button>

      <button
        onclick={clearLogs}
        class="px-3 py-1.5 rounded-xl bg-white/5 hover:bg-white/10 border border-white/10 text-neutral-200 hover:text-white text-xs font-bold flex items-center gap-1.5 transition-all cursor-pointer"
      >
        <Trash2 class="w-3.5 h-3.5" />
        <span>Clear</span>
      </button>
    </div>
  </div>

  <!-- Crash Detection Alert Banner -->
  {#if launcher.hasCrashDetected}
    <div
      class="rounded-xl border border-rose-500/40 bg-gradient-to-r from-rose-950/60 via-[#180a0d] to-[#12080a] p-4 flex flex-col sm:flex-row items-start sm:items-center justify-between gap-3 shadow-lg shrink-0"
    >
      <div class="flex items-center gap-3">
        <div
          class="w-9 h-9 rounded-lg bg-rose-500/20 border border-rose-500/30 flex items-center justify-center text-rose-400 shrink-0"
        >
          <AlertTriangle class="w-5 h-5" />
        </div>
        <div>
          <h4 class="font-bold text-sm text-white flex items-center gap-2">
            <span>Minecraft Crash or Fatal Exception Detected</span>
            <span
              class="px-2 py-0.5 rounded bg-rose-500/20 text-rose-300 text-[11px] font-bold uppercase"
            >
              Action Required
            </span>
          </h4>
          <p class="text-xs text-neutral-300 mt-0.5 leading-relaxed">
            The JVM recorded an error or crash. Use the button to copy the stack
            trace report directly for debugging.
          </p>
        </div>
      </div>

      <button
        onclick={copyCrashSnippet}
        class="px-4 py-2 rounded-xl bg-rose-600 hover:bg-rose-500 text-white font-bold text-xs uppercase tracking-wider flex items-center gap-2 transition-all cursor-pointer shrink-0 shadow-md shadow-rose-600/30"
      >
        {#if crashCopied}
          <Check class="w-3.5 h-3.5" />
          <span>Crash Report Copied!</span>
        {:else}
          <Copy class="w-3.5 h-3.5" />
          <span>1-Click Copy Crash Report</span>
        {/if}
      </button>
    </div>
  {/if}

  <!-- Log Filters & Live Search Bar -->
  <div
    class="flex flex-col sm:flex-row items-stretch sm:items-center justify-between gap-3 shrink-0"
  >
    <!-- Level Filters -->
    <div class="flex items-center gap-1.5 overflow-x-auto scrollbar-none">
      <span
        class="text-xs text-neutral-400 font-medium flex items-center gap-1 mr-1"
      >
        <Filter class="w-3 h-3" />
        <span>Level:</span>
      </span>

      <button
        onclick={() => (selectedLevel = "ALL")}
        class="px-2.5 py-1 rounded-lg text-xs font-bold transition-all cursor-pointer {selectedLevel ===
        'ALL'
          ? 'bg-white/15 text-white border border-white/25 shadow-sm'
          : 'bg-white/4 text-neutral-400 hover:text-white'}"
      >
        ALL ({launcher.consoleLogs.length})
      </button>

      <button
        onclick={() => (selectedLevel = "INFO")}
        class="px-2.5 py-1 rounded-lg text-xs font-bold transition-all cursor-pointer {selectedLevel ===
        'INFO'
          ? 'bg-blue-500/20 text-blue-300 border border-blue-500/40 shadow-sm'
          : 'bg-white/4 text-neutral-400 hover:text-blue-300'}"
      >
        INFO ({infoCount})
      </button>

      <button
        onclick={() => (selectedLevel = "WARN")}
        class="px-2.5 py-1 rounded-lg text-xs font-bold transition-all cursor-pointer {selectedLevel ===
        'WARN'
          ? 'bg-amber-500/20 text-amber-300 border border-amber-500/40 shadow-sm'
          : 'bg-white/4 text-neutral-400 hover:text-amber-300'}"
      >
        WARN ({warnCount})
      </button>

      <button
        onclick={() => (selectedLevel = "ERROR")}
        class="px-2.5 py-1 rounded-lg text-xs font-bold transition-all cursor-pointer {selectedLevel ===
        'ERROR'
          ? 'bg-rose-500/20 text-rose-300 border border-rose-500/40 shadow-sm'
          : 'bg-white/4 text-neutral-400 hover:text-rose-300'}"
      >
        ERROR ({errorCount})
      </button>
    </div>

    <!-- Live Search Input -->
    <div class="relative min-w-[240px]">
      <Search
        class="w-3.5 h-3.5 text-neutral-400 absolute left-3 top-1/2 -translate-y-1/2"
      />
      <input
        type="text"
        bind:value={searchQuery}
        placeholder="Search logs in real-time..."
        class="w-full pl-8 pr-7 py-1.5 rounded-lg bg-white/5 border border-white/10 focus:border-red-500/50 outline-none text-xs text-white placeholder-neutral-400 transition-all font-mono"
      />
      {#if searchQuery}
        <button
          onclick={() => (searchQuery = "")}
          class="absolute right-2.5 top-1/2 -translate-y-1/2 text-neutral-400 hover:text-white cursor-pointer"
        >
          <X class="w-3 h-3" />
        </button>
      {/if}
    </div>
  </div>

  <!-- Terminal Output Area -->
  <div
    bind:this={consoleBox}
    class="flex-1 rounded-2xl bg-[#06070a] border border-white/8 p-4 font-mono text-xs overflow-y-auto select-text space-y-1 shadow-inner leading-relaxed"
  >
    {#if launcher.consoleLogs.length === 0}
      <div
        class="h-full flex flex-col items-center justify-center text-neutral-500 space-y-2 select-none"
      >
        <Terminal class="w-8 h-8 opacity-40" />
        <span class="text-xs"
          >No log events yet. Launch Minecraft to observe live engine outputs or
          click "Load latest.log".</span
        >
      </div>
    {:else if displayedLogs.length === 0}
      <div
        class="h-full flex flex-col items-center justify-center text-neutral-500 space-y-2 select-none"
      >
        <Search class="w-8 h-8 opacity-40" />
        <span class="text-xs"
          >No log lines matching the current filter or search query.</span
        >
      </div>
    {:else}
      {#each displayedLogs as line}
        <div
          class="break-all font-mono text-[11px] leading-5 {getLogClass(line)}"
        >
          {line}
        </div>
      {/each}
    {/if}
  </div>
</div>
