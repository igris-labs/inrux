//! Caret line boxes independent of font ascent/descent overflow.
use parley::{Affinity, Brush, Cursor, Layout};
/// Return a UTF-8 caret rectangle using the logical line box, rather than overflowing font metrics.
pub fn caret_rect<B: Brush>(layout: &Layout<B>, index: usize, width: f32) -> [f32; 4] {
    let raw = Cursor::from_byte_index(layout, index, Affinity::Downstream).geometry(layout, width);
    let middle = (raw.y0 + raw.y1) as f32 * 0.5;
    let mut top = 0.0;
    for line in layout.lines() {
        let height = line.metrics().line_height;
        if middle >= top && middle <= top + height {
            return [raw.x0 as f32, top, width, height];
        }
        top += height;
    }
    [
        raw.x0 as f32,
        raw.y0 as f32,
        width,
        (raw.y1 - raw.y0) as f32,
    ]
}

/// Return visual selection rectangles using common logical line boxes, including bidirectional runs.
pub fn selection_rects<B: Brush>(
    layout: &Layout<B>,
    range: std::ops::Range<usize>,
) -> Vec<[f32; 4]> {
    let selection = parley::Selection::new(
        Cursor::from_byte_index(layout, range.start, Affinity::Downstream),
        Cursor::from_byte_index(layout, range.end, Affinity::Upstream),
    );
    let mut top = 0.0;
    let rows = layout
        .lines()
        .map(|line| {
            let row = (top, line.metrics().line_height);
            top += row.1;
            row
        })
        .collect::<Vec<_>>();
    selection
        .geometry(layout)
        .into_iter()
        .map(|(rect, index)| {
            let (y, height) = rows
                .get(index)
                .copied()
                .unwrap_or((rect.y0 as f32, (rect.y1 - rect.y0) as f32));
            [rect.x0 as f32, y, (rect.x1 - rect.x0) as f32, height]
        })
        .collect()
}

/// Underline a marked UTF-8 range at the font's underline position, split across visual runs.
pub fn underline_rects<B: Brush>(
    layout: &Layout<B>,
    range: std::ops::Range<usize>,
) -> Vec<[f32; 4]> {
    use parley::PositionedLayoutItem;
    let selection = parley::Selection::new(
        Cursor::from_byte_index(layout, range.start, Affinity::Downstream),
        Cursor::from_byte_index(layout, range.end, Affinity::Upstream),
    );
    selection
        .geometry(layout)
        .into_iter()
        .filter_map(|(rect, index)| {
            let line = layout.lines().nth(index)?;
            let offset = line
                .items()
                .find_map(|item| match item {
                    PositionedLayoutItem::GlyphRun(glyphs) => {
                        Some(glyphs.run().metrics().underline_offset)
                    }
                    _ => None,
                })
                .unwrap_or(0.0);
            Some([
                rect.x0 as f32,
                line.metrics().baseline - offset,
                (rect.x1 - rect.x0) as f32,
                1.0,
            ])
        })
        .collect()
}
