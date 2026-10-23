<script lang="ts">
  import { onMount } from "svelte";
  import { launcher } from "./state/launcher.svelte";

  // Layout components
  import Sidebar from "./components/layout/Sidebar.svelte";
  import Header from "./components/layout/Header.svelte";

  // Tab views
  import PlayTab from "./components/tabs/PlayTab.svelte";
  import ProfilesTab from "./components/tabs/ProfilesTab.svelte";
  import ModStudioTab from "./components/tabs/ModStudioTab.svelte";
  import SkinsTab from "./components/tabs/SkinsTab.svelte";
  import SettingsTab from "./components/tabs/SettingsTab.svelte";
  import ConsoleTab from "./components/tabs/ConsoleTab.svelte";

  // Modals & Overlays
  import AccountModal from "./components/modals/AccountModal.svelte";
  import CreateProfileModal from "./components/modals/CreateProfileModal.svelte";
  import Toast from "./components/common/Toast.svelte";

  onMount(async () => {
    await launcher.init();
  });
</script>

<div
  class="flex h-screen w-screen overflow-hidden bg-[#08090d] text-white select-none font-sans"
>
  <!-- Left Navigation Sidebar -->
  <Sidebar />

  <!-- Main Application Body -->
  <div
    class="flex-1 flex flex-col min-w-0 h-full overflow-hidden bg-gradient-to-br from-[#08090d] to-[#0d0e14]"
  >
    <!-- Top Bar Header -->
    <Header />

    <!-- Tab View Router -->
    <main class="flex-1 overflow-hidden relative">
      {#if launcher.activeTab === "play"}
        <PlayTab />
      {:else if launcher.activeTab === "instances"}
        <ProfilesTab />
      {:else if launcher.activeTab === "mods"}
        <ModStudioTab />
      {:else if launcher.activeTab === "skins"}
        <SkinsTab />
      {:else if launcher.activeTab === "settings"}
        <SettingsTab />
      {:else if launcher.activeTab === "console"}
        <ConsoleTab />
      {/if}
    </main>
  </div>

  <!-- Global Modals & Notifications -->
  <AccountModal />
  <CreateProfileModal />
  <Toast />
</div>
