export default defineAppConfig({
  indocio: {
    name: "Inrux",
    description: "A native GUI framework for Rust. Early alpha documentation.",
    customizer: { enabled: false },
    header: {
      links: [
        { label: "Guide", to: "/getting-started/introduction" },
        { label: "Components", to: "/components/overview" },
        { label: "Architecture", to: "/internals/architecture" },
        { label: "GitHub", to: "https://github.com/igris-labs/inrux" },
      ],
    },
    docs: {
      content: { width: "46rem" },
      sidebar: { width: 260 },
      toc: { position: "right", depth: 3 },
    },
  },
});
