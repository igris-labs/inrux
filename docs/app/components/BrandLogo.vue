<script setup lang="ts">
defineProps<{ wordmark?: boolean }>();
</script>

<template>
  <span class="inrux-brand">
    <img
      src="/brand/mark-light.svg"
      class="inrux-brand-light"
      width="28"
      height="28"
      alt=""
    />
    <img
      src="/brand/mark-dark.svg"
      class="inrux-brand-dark"
      width="28"
      height="28"
      alt=""
    />
    <span v-if="wordmark">Inrux</span>
  </span>
</template>
