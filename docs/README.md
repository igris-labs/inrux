# Inrux documentation website

This folder contains the public Inrux documentation site at [inrux.igrislabs.com](https://inrux.igrislabs.com). It uses Nuxt 4, Nuxt Content 3 and the published `@igrislabs/indocio` layer for navigation, search, Markdown components and light/dark mode. It does not require a separate Indocio checkout.

## Run locally

Use Node.js 22.19 or newer and pnpm 10.7.1. From the repository root:

```sh
pnpm --dir docs install --frozen-lockfile
pnpm --dir docs dev
```

Open the local URL printed by Nuxt. Building Rust applications does not require this website or its Node dependencies.

## Read and edit

Write pages in `content/`. Numeric filename prefixes set navigation order and disappear from page URLs. Component pages omit these prefixes so navigation sorts them alphabetically by filename. Each page supplies title, description and a `navigation.icon` value from the existing Tabler outline set. Use a current-color icon that describes the page. The application stylesheet sets the leading slot to 16px; Indocio handles navigation and active states. `.navigation.yml` names a sidebar section; `content/index.yml` configures the landing page.

- [Getting started](content/1.getting-started/01.introduction.md)
- [The .iru format and its tradeoffs](content/3.authoring/01.iru-format.md)
- [Components](content/5.components/overview.md)
- [Styling](content/4.styling/01.overview.md)
- [Architecture](content/2.internals/01.architecture.md)
- [Editing and building this website](content/6.tooling/03.documentation.md)

`app/app.config.ts` configures navigation and layout. `app/components/BrandLogo.vue` uses the Inrux mark. `app/assets/main.css` sets the brand colors and the Inter/JetBrains Mono font choices. `public/diagrams/inrux-architecture.svg` mirrors the root architecture image. The architecture guide explains its layers and marks planned work; `ArchitectureDiagram.vue` supplies the smaller compilation/thread diagrams; `ProsePre.vue` maps standalone Rust fences to the Rust grammar. `app/utils/highlight-iru.ts` highlights the Rust and CSS bodies inside complete `.iru` examples with their own grammars while keeping markup highlighting for the view. The small `InruxCodeBlock.vue` override keys highlighted data by source so client-side navigation cannot reuse another page's code. The Indocio layer owns the remaining UI and content collections. `app/components/docs/DocsSidebar.vue` wraps the layer sidebar and reveals the active link after hydration, route changes, mobile drawer opening and viewport changes. It moves only the sidebar scrollport; unrelated page interactions preserve manual sidebar scrolling. The installation guide uses the layer's accessible `tabs` and `tab-item` Markdown components.

## Validate and build

```sh
pnpm --dir docs test
pnpm --dir docs typecheck
pnpm --dir docs build
```

To check the copy-and-run component examples, also run:

```sh
python3 docs/scripts/check-examples.py
```

On Windows, run the same script with `py -3 docs/scripts/check-examples.py`.

This separate check needs Rust and the native [build prerequisites](content/1.getting-started/02.installation.md). It creates the real CLI starter in a temporary directory, pastes each complete `App.iru` into it, checks Rust types/styles/bindings, and validates the initial scene with egui. It enables `markdown` for the Markdown sample. It does not open native windows or certify every interaction on every engine. The ordinary website build remains Node-only.

Static output is `docs/.output/public/`. Serve it with a static host. Use `pnpm --dir docs build` for a Nitro server build and `pnpm --dir docs preview` to preview a build. Generated files and dependencies stay out of Git.

The default public origin is `https://inrux.igrislabs.com`. Canonical URLs, Open Graph assets, robots metadata and the sitemap use that origin. Set `NUXT_SITE_URL` before building only when you need a different preview/deployment origin. The current site assumes deployment at the origin root. A URL-prefix deployment needs its asset paths and route base configured and tested first.

No deployment runs during development. Documentation code is MIT; bundled fonts retain the [SIL Open Font License](public/fonts/NotoSans-OFL.txt). Keep the matching Mono notice too. See [project notices](../THIRD_PARTY_NOTICES.md).

## Dependency maintenance

See [current versions and migration notes](content/6.tooling/05.dependencies.md). The site aligns Nuxt Content across its app and layer to keep generated collection types consistent. TypeScript remains at 6.0.3 for Vue typechecking; the extension has its own TypeScript 7 toolchain.
