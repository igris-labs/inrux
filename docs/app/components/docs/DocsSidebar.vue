<script setup lang="ts">
import type { ContentNavigationItem } from "@nuxt/content";
import type { DocsConfig } from "@igrislabs/indocio/types";
import LayerDocsSidebar from "#layers/indocio/app/components/docs/DocsSidebar.vue";
import { nextTick, onBeforeUnmount, onMounted, ref, watch } from "vue";
import { revealNavigation } from "~/utils/reveal-navigation";

const props = defineProps<{
  tree: ContentNavigationItem[];
  settings: DocsConfig;
}>();
const collapsed = defineModel<boolean>("collapsed", { default: false });
const sidebar = ref<InstanceType<typeof LayerDocsSidebar>>();
const route = useRoute();
let mounted = false;
let frame: number | undefined;

async function reveal() {
  if (!mounted) return;
  // The layer restores scroll in a post-flush watcher, including after hydration.
  await nextTick();
  if (!mounted || frame !== undefined) return;
  frame = requestAnimationFrame(() => {
    frame = undefined;
    for (const navigation of document.querySelectorAll<HTMLElement>(
      '[aria-label="Documentation navigation"]',
    )) {
      revealNavigation(navigation, route.path);
    }
  });
}
function toggle() {
  sidebar.value?.toggle();
  void reveal();
}
defineExpose({ toggle });
watch([() => route.path, () => props.tree], reveal, { flush: "post" });
onMounted(() => {
  mounted = true;
  window.addEventListener("resize", reveal, { passive: true });
  void reveal();
});
onBeforeUnmount(() => {
  mounted = false;
  window.removeEventListener("resize", reveal);
  if (frame !== undefined) cancelAnimationFrame(frame);
});
</script>

<template>
  <LayerDocsSidebar
    ref="sidebar"
    v-model:collapsed="collapsed"
    :tree="tree"
    :settings="settings"
  >
    <template v-for="(_, name) in $slots" #[name]="scope">
      <slot :name="name" v-bind="scope ?? {}" />
    </template>
  </LayerDocsSidebar>
</template>
