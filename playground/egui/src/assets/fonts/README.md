# Bundled fonts

Every playground includes the same font bytes and registers their intrinsic family names in main.rs.

- `NotoSans-Regular.ttf`: Noto Sans, existing OFL license in OFL.txt.
- `NotoSansMono.ttf`: Noto Sans Mono variable TrueType face, unmodified, downloaded November 8, 2026 from [Google Fonts](https://github.com/google/fonts/blob/main/ofl/notosansmono/NotoSansMono%5Bwdth%2Cwght%5D.ttf). License: NotoSansMono-OFL.txt. Used for preformatted Markdown code via `.md-code-text`.

The fonts are embedded at build time; runtime use requires no font download or system installation. Bundling the code face increases the application asset/binary footprint; it is a playground asset, not a mandatory dependency of inrux-markdown.
