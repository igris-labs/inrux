# Repository structure

The repository is a Cargo workspace. Framework libraries live under `packages/`, developer tooling under `tooling/`, and independent example applications under `playground/`.

```text
inrux/
├── Cargo.toml
├── Cargo.lock
├── LICENSE
├── THIRD_PARTY_NOTICES.md
├── README.md
├── CONTRIBUTING.md
├── docs/                       Public Markdown documentation
├── assets/brand/               Logos, palette and usage guide
├── licenses/                   Third-party license texts
├── packages/
│   ├── inrux/                  Public application facade
│   ├── inrux-core/             State, components, events, scopes, tasks
│   ├── inrux-components/       Engine-independent component descriptions
│   ├── inrux-style/            Typed styles, CSS parsing and indexed matching
│   ├── inrux-text/             Shared shaping, geometry and glyph rasterization
│   ├── inrux-adapter/          Engine-neutral host/service contracts and policy
│   ├── inrux-egui/             Private egui binding
│   ├── inrux-masonry/          Private Masonry binding
│   ├── inrux-gpui/             Private GPUI binding
│   ├── inrux-compiler/         .iru compilation and source metadata
│   ├── inrux-cli/              cargo-inrux executable and starter templates
│   ├── inrux-language-server/  Rust editor protocol server
│   ├── inrux-markdown/         Optional Markdown-to-component rendering
│   └── inrux-devtools/         Optional prototype inspector
├── playground/
│   ├── egui/
│   ├── masonry/
│   └── gpui/
├── tooling/
│   ├── inrux-vscode/          VS Code extension
│   ├── backend-check/         Backend dependency-boundary checks
│   └── playground-check/      Portable source/font consistency checks
├── vendor/                    Reviewed upstream compatibility patches
└── .github/workflows/         CI configuration
```

Each crate/tool has a focused README. Cargo workspace membership uses `packages/*` and `playground/*`; TypeScript editor tooling stays outside Cargo's package graph. Application code normally imports the `inrux` facade and uses `inrux-compiler` as a build dependency.

## Application convention

Every playground owns its source and assets. All executable code lives under `src/`; `build.rs` stays at the project root as required by Cargo.

- `src/main.rs`: entry point and mount configuration.
- `src/App.iru`: root component and application composition.
- `src/components/`: reusable application components.
- `src/views/`: screens/windows.
- `src/layouts/`: reusable shells using default child content.
- `src/composables/`: ordinary Rust state/logic helpers.
- `src/styles/`: application styles and theme tokens.
- `src/assets/`: fonts and other application assets, with notices.

Layouts and composables are conventions built from ordinary component/Rust facilities. They do not require routing or a special composable runtime. Framework component implementations use focused modules under `components/<component-name>/`; see [contributor rules](../CONTRIBUTING.md).

## Dependency boundaries

Core, components, styles and adapter contracts do not expose engine widget types. Each backend owns its native resources and translation. Build/editor tools consume compiler metadata rather than duplicating language rules. New utilities should live in the module/crate that owns their behavior; create another package when it has a meaningful independent boundary.
