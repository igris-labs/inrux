<script setup lang="ts">
const props = withDefaults(
  defineProps<{ kind?: "framework" | "compile" | "runtime" }>(),
  { kind: "framework" },
);
const diagrams = {
  framework: {
    caption:
      "One selected native binding sits below shared framework contracts. The current bindings share Taffy layout and Parley/Swash text.",
    steps: [
      [
        "Your application",
        "main.rs mounts App.iru; ordinary Rust modules hold services and business rules.",
      ],
      [
        "The inrux facade",
        "Components, typed props/events, state, provide/inject and application styles.",
      ],
      [
        "inrux-adapter",
        "Engine-neutral windows, capability checks, layout/text services and common input policy.",
      ],
      [
        "One native binding",
        "inrux-egui, inrux-masonry or inrux-gpui owns its engine objects and native uploads.",
      ],
      [
        "Native application window",
        "egui, Masonry or GPUI supplies the selected window/rendering integration.",
      ],
    ],
  },
  compile: {
    caption:
      "Cargo compiles generated Rust together with your Rust modules. Runtime-selected imported candidates remain reachable.",
    steps: [
      [
        "Source files",
        "App.iru, explicit component imports and style blocks or precompiled standalone CSS.",
      ],
      [
        "inrux-compiler",
        "Validate syntax/imports; produce Rust code, typed style rules and editor source metadata.",
      ],
      [
        "Cargo and rustc",
        "Type-check props/handlers and compile generated code with normal dependencies.",
      ],
      [
        "Native binary",
        "Link the selected GUI backend and the reachable application code.",
      ],
    ],
  },
  runtime: {
    caption:
      "Events and frame descriptions cross bounded queues. Slow work can still stall its owning thread.",
    steps: [
      [
        "UI thread",
        "Receive native events; manage layout, input, painting and native objects.",
      ],
      [
        "Application coordinator",
        "Process queued handlers; update state and computed dependencies; publish component descriptions.",
      ],
      [
        "Bounded task pool",
        "Run expensive work with owned inputs and cooperative cancellation.",
      ],
      [
        "Typed completion to coordinator",
        "Apply results only while the owning mount/generation remains valid.",
      ],
      [
        "Newest description to UI thread",
        "Translate shared geometry and glyph pixels into native drawing and controls.",
      ],
    ],
  },
} as const;
const diagram = computed(() => diagrams[props.kind]);
</script>

<template>
  <figure class="inrux-diagram">
    <ol>
      <li v-for="[title, detail] in diagram.steps" :key="title">
        <strong>{{ title }}</strong>
        <small>{{ detail }}</small>
      </li>
    </ol>
    <figcaption>{{ diagram.caption }}</figcaption>
  </figure>
</template>
