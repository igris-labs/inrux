<script setup lang="ts">
import { highlightIru } from "~/utils/highlight-iru";
const props = withDefaults(
  defineProps<{
    code: string;
    header?: boolean;
    filename?: string;
    language?: string;
    preClass?: string;
    preAttrs?: Record<string, unknown>;
  }>(),
  { header: true },
);
const status = ref("Copy code");
// A component ID can repeat across page payloads. Key highlighted data by its source.
const { data: highlighted } = await useAsyncData(
  () => `inrux-code-sfc:${props.language ?? "plain"}:${props.code}`,
  () => {
    const language = props.language ?? "plain";
    const component =
      language === "iru" ||
      props.filename?.endsWith(".iru") ||
      (language === "xml" &&
        /^[ \t]*<rust(?:[ \t]+explicit)?[ \t]*>/m.test(props.code));
    if (component) return highlightIru(props.code, highlightCode);
    // XML's grammar rejects @events; the markup grammar recognizes those attributes.
    return highlightCode(
      props.code,
      language === "xml" ? "vue" : language === "rust" ? "rs" : language,
    );
  },
);
watch(
  () => props.code,
  () => {
    status.value = "Copy code";
  },
);
async function copy() {
  try {
    await navigator.clipboard.writeText(props.code);
    status.value = "Copied";
  } catch {
    status.value = "Select and copy the code below";
  }
}
</script>

<template>
  <div
    class="group/code relative my-6 min-w-0 overflow-hidden rounded-lg border border-border/60 bg-(--syntax-background) text-sm syntax-highlight"
  >
    <div
      :class="
        header
          ? 'flex min-h-10 items-center justify-between gap-3 border-b border-border/50 bg-(--syntax-header) px-4'
          : 'absolute top-2 right-2 z-20'
      "
    >
      <span
        v-if="header"
        class="min-w-0 truncate text-xs text-muted-foreground"
        >{{ filename ?? language ?? "Code" }}</span
      >
      <UiButton
        variant="ghost"
        color="secondary"
        size="xs"
        shape="icon"
        :aria-label="status"
        @click="copy"
      >
        <Icon
          :name="status === 'Copied' ? 'lucide:check' : 'lucide:copy'"
          class="size-3.5"
          aria-hidden="true"
        />
      </UiButton>
      <span class="sr-only" role="status">{{
        status === "Copy code" ? "" : status
      }}</span>
    </div>
    <pre
      v-bind="preAttrs"
      :class="[preClass, !header && 'pr-12']"
      class="m-0 max-h-120 overflow-auto bg-transparent px-5 py-5 font-mono text-[13px] leading-6 text-(--syntax-foreground)"
    ><code v-if="highlighted" v-html="highlighted" /><code v-else>{{ code }}</code></pre>
  </div>
</template>
