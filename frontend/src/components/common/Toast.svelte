<script lang="ts">
  import { launcher } from "../../state/launcher.svelte";
  import { Sparkles, Info } from "lucide-svelte";
</script>

{#if launcher.toastMessage}
  <div class="fixed bottom-6 right-6 z-50 flex items-center gap-3 px-4 py-3 rounded-2xl bg-[#11131c]/95 border border-red-500/30 text-white shadow-2xl backdrop-blur-xl animate-in fade-in slide-in-from-bottom-3 duration-200">
    <div class="w-2 h-2 rounded-full bg-red-500 animate-pulse"></div>
    <span class="text-xs font-mono font-medium">{launcher.toastMessage}</span>
  </div>
{/if}
