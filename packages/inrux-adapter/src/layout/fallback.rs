//! Taffy-only implementation, omitted when the optional fallback is disabled.
use super::{LayoutBox, TextMeasure};
use crate::{
    AdapterError,
    layout_policy::{LayoutNode, Measure, build_layout},
};
use inrux_components::Node;
use taffy::prelude as tf;
/// Bridge fallible native measurements into Taffy's infallible callback without publishing partial geometry.
pub fn compute(
    node: &Node,
    width: f32,
    height: Option<f32>,
    measure: &mut impl crate::services::text::TextMeasurer,
) -> Result<LayoutBox, AdapterError> {
    let mut failure = None;
    let mut tree = tf::TaffyTree::<Measure>::new();
    let root = build_layout(node, &mut tree).map_err(layout_error)?;
    tree.compute_layout_with_measure(
        root.id,
        tf::Size {
            width: tf::AvailableSpace::Definite(width),
            height: height.map_or(tf::AvailableSpace::MaxContent, tf::AvailableSpace::Definite),
        },
        |known, available, _, context, _| {
            let Some(context) = context else {
                return tf::Size::ZERO;
            };
            let max_width = known.width.unwrap_or(match available.width {
                // Intrinsic probes may subtract padding from zero available space.
                // Text providers receive an empty content box, never a negative width.
                tf::AvailableSpace::Definite(width) => {
                    if width < 0.0 {
                        0.0
                    } else {
                        width
                    }
                }
                tf::AvailableSpace::MinContent => 1.0,
                tf::AvailableSpace::MaxContent => f32::INFINITY,
            });
            if failure.is_some() {
                return tf::Size::ZERO;
            }
            let request = TextMeasure {
                text: &context.text,
                font_size: context.font_size,
                font_family: context.font_family.as_deref(),
                max_width: if context.control || context.preformatted {
                    f32::INFINITY
                } else {
                    max_width
                },
            };
            let result = crate::services::text::validate_request(&request)
                .and_then(|()| measure.measure(request))
                .and_then(|size| {
                    crate::services::text::validate_size(size, "taffy text measurer")?;
                    Ok(size)
                });
            let crate::services::text::TextSize { width, height } = match result {
                Ok(size) => size,
                Err(error) => {
                    failure = Some(error);
                    return tf::Size::ZERO;
                }
            };
            tf::Size {
                width: if context.preformatted {
                    width.ceil()
                } else {
                    known.width.unwrap_or(if context.input {
                        220.0
                    } else {
                        // Native layout may quantize bounds; round natural size up so a final word fits.
                        (width
                            + if context.control {
                                context.control_extra
                            } else {
                                0.0
                            })
                        .ceil()
                    })
                },
                height: known.height.unwrap_or(height.ceil()),
            }
        },
    )
    .map_err(layout_error)?;
    if let Some(error) = failure {
        return Err(error);
    }
    geometry(&root, &tree)
}
/// Copy resolved geometry out of Taffy so its IDs/error types stay private.
fn geometry(node: &LayoutNode, tree: &tf::TaffyTree<Measure>) -> Result<LayoutBox, AdapterError> {
    let layout = tree.layout(node.id).map_err(layout_error)?;
    Ok(LayoutBox {
        content_width: layout.content_size.width,
        content_height: layout.content_size.height,
        x: layout.location.x,
        y: layout.location.y,
        width: layout.size.width,
        height: layout.size.height,
        children: node
            .children
            .iter()
            .map(|child| geometry(child, tree))
            .collect::<Result<_, _>>()?,
    })
}
/// Convert implementation errors without exposing a layout library in public signatures.
fn layout_error(error: taffy::TaffyError) -> AdapterError {
    AdapterError::Application(format!("Layout failed: {error}"))
}
