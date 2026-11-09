# Inrux brand kit

The identity follows the supplied orange/black logo concept: two folded ribbons surround an open window. The open center represents a native surface; the folded layers suggest composition. The tagline is **A native GUI framework for Rust.**

These are editable SVG redraws based on the supplied raster reference, not the original logo master. The wordmark is a new outlined interpretation; review it before treating this kit as the final identity. It uses modified Noto Sans outlines and requires no installed fonts to render.

## Assets

| Asset | Light surface | Dark surface |
| --- | --- | --- |
| Mark | [Color](inrux-mark-light.svg) | [Color](inrux-mark-dark.svg) |
| Monochrome mark | [Black](inrux-mark-mono-light.svg) | [White](inrux-mark-mono-dark.svg) |
| Wordmark | [Black](inrux-wordmark-light.svg) | [White](inrux-wordmark-dark.svg) |
| Horizontal lockup | [Light](inrux-lockup-light.svg) | [Dark](inrux-lockup-dark.svg) |

[Social card SVG](inrux-social.svg) is 1280 × 640; its [PNG export](inrux-social.png) is suitable for a GitHub social preview. [JSON tokens](tokens.json) and [CSS tokens](tokens.css) record the same palette. Brand tokens are optional documentation/design assets; they do not impose styles on headless components.

## Color

| Role | Value | Use |
| --- | --- | --- |
| Inrux orange | `#FF671D` | Ribbon, large graphics and accents |
| Ink | `#111111` | Light-surface wordmark and primary text |
| White | `#FFFFFF` | Light surfaces and reversed mark |
| Muted | `#666666` | Secondary text on white |
| Line | `#E5E5E5` | Dividers on white |
| Orange text | `#B84200` | Accessible orange text on white |

Bright orange is an accent, not small text on white. Use ink text on an orange fill, or the darker orange for text on white. Verify contrast for actual text size/state/background. Avoid gradients or shadows inside the logo.

## Typography and layout

Use Noto Sans for documentation/application text and Noto Sans Mono for code. Keep the fixed outlined wordmark rather than retyping it in a substitute font. Body copy should be readable and left aligned; reserve the mark's irregular silhouette as the distinctive element.

Leave clear space of at least one quarter of the mark width around a standalone mark; use at least the height of the lowercase wordmark letters around a lockup. At small sizes use the mark alone: recommended minimum 24 px. Wordmark minimum width is 100 px; the full tagline lockup needs at least 540 px to remain legible. Do not stretch, rotate, crop or recolor individual layers. Preserve the transparent center. Use monochrome assets where the orange ribbon cannot reproduce reliably.

Use the social card for launch announcements with the explicit experimental-alpha label. The tagline describes project intent; it is not a stability or performance claim. Brand assets do not indicate endorsement by the project.

## License and attribution

Original artwork is covered by the repository's [MIT license](../../LICENSE). Letter outlines derive from Noto Sans; preserve [FONT-LICENSE.txt](FONT-LICENSE.txt), including the SIL Open Font License and copyright notices, when redistributing the derived lettering. See [third-party notices](../../THIRD_PARTY_NOTICES.md).
