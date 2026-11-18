<script setup lang="ts">
defineOptions({ inheritAttrs: false });
const props = defineProps<{
  code?: string;
  language?: string;
  filename?: string;
  highlights?: number[];
  meta?: string;
  class?: string;
}>();
// Indocio's highlighter names the Rust grammar "rs".
const language = computed(() =>
  props.language === "rust" ? "rs" : props.language,
);
</script>

<template>
  <InruxCodeBlock
    :code="code ?? ''"
    :filename="filename"
    :language="language"
    :pre-class="$props.class"
    :pre-attrs="$attrs"
  />
</template>
