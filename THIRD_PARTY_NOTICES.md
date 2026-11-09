# Third-party notices

Original Inrux code is licensed under the [MIT license](LICENSE). This does not relicense dependencies, adapted source or fonts. Their own license terms and copyright notices continue to apply. Cargo dependencies retain the licenses recorded by their upstream packages; consult the resolved dependency graph when distributing binaries.

## Adapted Apache-2.0 source

- **Masonry 0.4.0:** private input/text-area and label code in `packages/inrux-masonry/src/widgets/`, adapted from the Xilem/Druid authors. Copyright headers remain in the source. See [the adapter NOTICE](packages/inrux-masonry/NOTICE).
- **GPUI 0.2.2:** private input code in `packages/inrux-gpui/src/widgets/input/`, adapted from Zed's input example, Copyright Zed Industries, Inc. See [the adapter NOTICE](packages/inrux-gpui/NOTICE).

The [Apache-2.0 text](licenses/Apache-2.0.txt) is retained for these adaptations. These two crates use `MIT AND Apache-2.0` package metadata to describe mixed contents; this is not a dual-license offer for the project. Original Inrux files remain MIT; Apache-marked adaptations retain Apache-2.0.

## Fonts

Noto Sans Regular and Noto Sans Mono are included under the SIL Open Font License 1.1. Keep the relevant copyright/license files when redistributing them:

- [Shared text font license](packages/inrux-text/assets/OFL.txt).
- [Playground Noto Sans license](playground/egui/src/assets/fonts/OFL.txt).
- [Playground Noto Sans Mono license](playground/egui/src/assets/fonts/NotoSansMono-OFL.txt).

The other two playgrounds contain matching font assets and license copies. The brand wordmark and social graphics use outlines derived from Noto Sans; the brand kit includes the same font license notice. No system font files are redistributed by the kit.

## Local upstream patches

`vendor/` contains compatibility patches for `block` 0.1.6 (MIT) and `proc-macro-error2` 2.0.1 (MIT OR Apache-2.0). Their metadata, source headers and supplied license texts are retained. See [vendor details](vendor/README.md). Do not remove the Cargo patch entries without checking the affected upstream future-incompatibility fixes.
