//! Native text metrics retained only for requests made by the current layout, never navigation history.
use inrux_adapter::{layout::TextMeasure, services::text::TextSize};
use std::collections::HashMap;
#[derive(Default)]
pub(crate) struct Measurements {
    entries: HashMap<String, Vec<Measurement>>,
}
struct Measurement {
    font_size: u32,
    max_width: u32,
    family: Option<String>,
    size: TextSize,
}
impl Measurements {
    /// Reuse matching logical font/text/wrap metrics; paint colors do not affect shaping.
    pub(crate) fn get(&self, request: &TextMeasure<'_>) -> Option<TextSize> {
        self.entries
            .get(request.text)?
            .iter()
            .find(|entry| {
                entry.font_size == request.font_size.to_bits()
                    && entry.max_width == request.max_width.to_bits()
                    && entry.family.as_deref() == request.font_family
            })
            .map(|entry| entry.size)
    }
    /// Own each distinct text once per active request set; unused previous-page entries are dropped.
    pub(crate) fn insert(&mut self, request: &TextMeasure<'_>, size: TextSize) {
        self.entries
            .entry(request.text.to_owned())
            .or_default()
            .push(Measurement {
                font_size: request.font_size.to_bits(),
                max_width: request.max_width.to_bits(),
                family: request.font_family.map(str::to_owned),
                size,
            });
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    /// Font size/family, text and wrapping all participate in metric identity.
    #[test]
    fn metrics_do_not_alias_changed_text_fonts_or_widths() {
        let request = TextMeasure {
            text: "Label",
            font_size: 13.0,
            font_family: Some("Noto Sans"),
            max_width: 100.0,
        };
        let mut cache = Measurements::default();
        cache.insert(
            &request,
            TextSize {
                width: 35.0,
                height: 16.0,
            },
        );
        assert_eq!(cache.get(&request).unwrap().width, 35.0);
        for changed in [
            TextMeasure {
                text: "Other",
                ..request
            },
            TextMeasure {
                font_size: 14.0,
                ..request
            },
            TextMeasure {
                font_family: None,
                ..request
            },
            TextMeasure {
                max_width: 200.0,
                ..request
            },
        ] {
            assert!(cache.get(&changed).is_none());
        }
    }
}
