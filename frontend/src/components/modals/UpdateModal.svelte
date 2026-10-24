<script lang="ts">
  import { launcher } from "../../state/launcher.svelte";
  import {
    X,
    Sparkles,
    Download,
    ExternalLink,
    Calendar,
    ArrowRight,
    CheckCircle2,
  } from "lucide-svelte";

  function formatDate(iso: string | null | undefined): string {
    if (!iso) return "Recently published";
    try {
      const d = new Date(iso);
      return d.toLocaleDateString(undefined, {
        year: "numeric",
        month: "short",
        day: "numeric",
      });
    } catch {
      return iso;
    }
  }

  function handleDownload() {
    launcher.openReleaseUrl();
    launcher.showUpdateModal = false;
  }

  function handleViewOnGithub() {
    if (launcher.updateInfo?.html_url) {
      launcher.openFolder(launcher.updateInfo.html_url);
    }
    launcher.showUpdateModal = false;
  }
</script>

{#if launcher.showUpdateModal && launcher.updateInfo}
  <div
    class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/85 backdrop-blur-md animate-in fade-in duration-200"
  >
    <div
      class="relative w-full max-w-2xl rounded-3xl bg-[#0d1017] border border-white/12 shadow-2xl p-6 sm:p-7 space-y-6 flex flex-col max-h-[90vh]"
    >
      <!-- Modal Header -->
      <div
        class="flex items-center justify-between pb-4 border-b border-white/8 shrink-0"
      >
        <div class="flex items-center gap-3.5">
          <div
            class="w-11 h-11 rounded-2xl bg-gradient-to-br from-emerald-500/20 to-teal-500/20 border border-emerald-500/30 flex items-center justify-center text-emerald-400 shrink-0 shadow-lg shadow-emerald-500/10"
          >
            <Sparkles class="w-5 h-5" />
          </div>
          <div>
            <h3 class="text-lg font-black uppercase text-white tracking-tight flex items-center gap-2">
              <span>Software Update Available</span>
              <span
                class="px-2 py-0.5 rounded-full bg-emerald-500/20 text-emerald-300 text-[10px] font-black uppercase tracking-wider"
              >
                {launcher.updateInfo.latest_version}
              </span>
            </h3>
            <p class="text-xs text-neutral-300 mt-0.5">
              An updated build of Digital Vigital Launcher is ready to download.
            </p>
          </div>
        </div>

        <button
          onclick={() => (launcher.showUpdateModal = false)}
          class="p-2.5 rounded-xl bg-white/5 hover:bg-white/10 text-neutral-400 hover:text-white transition-all cursor-pointer"
          title="Close modal"
        >
          <X class="w-4 h-4" />
        </button>
      </div>

      <!-- Version & Release Info Card -->
      <div
        class="p-4 rounded-2xl bg-gradient-to-r from-emerald-950/40 via-[#101e18]/40 to-transparent border border-emerald-500/25 flex flex-col sm:flex-row items-start sm:items-center justify-between gap-4 shrink-0"
      >
        <div class="flex items-center gap-3">
          <div class="text-center sm:text-left">
            <span class="text-[11px] font-bold uppercase tracking-wider text-neutral-400 block">Current Version</span>
            <span class="text-sm font-mono font-bold text-neutral-300">v{launcher.updateInfo.current_version}</span>
          </div>

          <div class="w-8 h-8 rounded-full bg-white/5 border border-white/10 flex items-center justify-center text-neutral-400 shrink-0">
            <ArrowRight class="w-3.5 h-3.5" />
          </div>

          <div class="text-center sm:text-left">
            <span class="text-[11px] font-bold uppercase tracking-wider text-emerald-400 block">New Version</span>
            <span class="text-sm font-mono font-black text-emerald-300">{launcher.updateInfo.latest_version}</span>
          </div>
        </div>

        <div class="flex items-center gap-2 text-xs text-neutral-400">
          <Calendar class="w-3.5 h-3.5" />
          <span>{formatDate(launcher.updateInfo.published_at)}</span>
        </div>
      </div>

      <!-- Release Notes Scroll Area -->
      <div class="flex-1 min-h-0 flex flex-col space-y-2">
        <div class="flex items-center justify-between text-xs font-bold uppercase tracking-wider text-neutral-400 shrink-0">
          <span>Release Notes & Highlights</span>
          <span class="text-neutral-500 font-normal lowercase">{launcher.updateInfo.release_name}</span>
        </div>

        <div
          class="flex-1 overflow-y-auto p-4 rounded-2xl bg-black/40 border border-white/8 text-xs text-neutral-200 leading-relaxed font-sans whitespace-pre-wrap select-text scrollbar-thin"
        >
          {#if launcher.updateInfo.release_notes && launcher.updateInfo.release_notes.trim()}
            {launcher.updateInfo.release_notes}
          {:else}
            <div class="text-neutral-500 italic py-6 text-center">
              No release notes provided for this version.
            </div>
          {/if}
        </div>
      </div>

      <!-- Action Buttons Footer -->
      <div
        class="pt-4 border-t border-white/8 flex flex-col sm:flex-row items-center justify-between gap-3 shrink-0"
      >
        <button
          onclick={handleViewOnGithub}
          class="w-full sm:w-auto px-4 py-2.5 rounded-xl bg-white/5 hover:bg-white/10 border border-white/10 text-neutral-200 hover:text-white text-xs font-bold flex items-center justify-center gap-2 transition-all cursor-pointer"
        >
          <ExternalLink class="w-3.5 h-3.5" />
          <span>View on GitHub</span>
        </button>

        <div class="flex items-center gap-2.5 w-full sm:w-auto justify-end">
          <button
            onclick={() => (launcher.showUpdateModal = false)}
            class="px-4 py-2.5 rounded-xl bg-white/5 hover:bg-white/10 text-neutral-300 hover:text-white text-xs font-bold transition-all cursor-pointer"
          >
            Later
          </button>

          <button
            onclick={handleDownload}
            class="flex-1 sm:flex-none px-6 py-2.5 rounded-xl bg-gradient-to-r from-emerald-600 to-teal-600 hover:from-emerald-500 hover:to-teal-500 text-white text-xs font-black uppercase tracking-wider flex items-center justify-center gap-2 transition-all cursor-pointer shadow-lg shadow-emerald-600/30 active:scale-95"
          >
            <Download class="w-4 h-4" />
            <span>Download Update</span>
          </button>
        </div>
      </div>
    </div>
  </div>
{/if}