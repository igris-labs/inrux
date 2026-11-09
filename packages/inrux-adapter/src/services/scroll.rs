//! Shared viewport policy; bindings provide native clipping, focus and logical input deltas.
use super::clipping::ClipRect;
use crate::{AdapterError, layout::LayoutBox};
use inrux_components::{Node, ScrollMetrics, ScrollOffset};
use inrux_style::{Overflow, Style};
/// Shared logical-pixel line/key distance; native pixel gestures retain their precision.
pub const LINE_STEP: f32 = 40.0;
/// Overlay scrollbar hit target width; no layout gutter is reserved.
pub const BAR_SIZE: f32 = 10.0;
/// Keyboard commands independent of native event types.
#[derive(Clone, Copy, Debug)]
pub enum ScrollKey {
    /// Move left by one line.
    Left,
    /// Move right by one line.
    Right,
    /// Move up by one line.
    Up,
    /// Move down by one line.
    Down,
    /// Move up by one viewport, with overlap.
    PageUp,
    /// Move down by one viewport, with overlap.
    PageDown,
    /// Start of the primary axis.
    Home,
    /// End of the primary axis.
    End,
}
impl ScrollKey {
    /// Normalize native key names at the binding boundary.
    pub fn from_name(name: &str) -> Option<Self> {
        Some(match name {
            "ArrowLeft" | "left" => Self::Left,
            "ArrowRight" | "right" => Self::Right,
            "ArrowUp" | "up" => Self::Up,
            "ArrowDown" | "down" => Self::Down,
            "PageUp" | "pageup" => Self::PageUp,
            "PageDown" | "pagedown" => Self::PageDown,
            "Home" | "home" => Self::Home,
            "End" | "end" => Self::End,
            _ => return None,
        })
    }
}
/// Native-independent scrollbar geometry, shared by painting and pointer handling.
#[derive(Clone, Copy, Debug)]
pub struct ScrollBar {
    /// True for a horizontal bar.
    pub horizontal: bool,
    /// Pointer target/track in local viewport coordinates.
    pub track: ClipRect,
    /// Current thumb in local viewport coordinates.
    pub thumb: ClipRect,
}
/// UI-owned fallback provider. No native resource or application state lives here.
#[derive(Clone, Debug, Default)]
pub struct ScrollState {
    mount: Option<(
        Option<String>,
        std::mem::Discriminant<inrux_components::NodeKind>,
    )>,
    metrics: ScrollMetrics,
    axes: (Overflow, Overflow),
    revision: Option<u64>,
    published: Option<ScrollMetrics>,
    drag: Option<(bool, f32)>,
    focus: Option<String>,
    indicators: super::indicators::Indicators,
}
impl ScrollState {
    /// Synchronize selected-provider geometry and apply an explicit command once per revision.
    pub fn update(&mut self, node: &Node, bounds: &LayoutBox) {
        let kind = std::mem::discriminant(&node.kind);
        if !self.mount.as_ref().is_some_and(|(generation, previous)| {
            generation == &node.generation && *previous == kind
        }) {
            // A remount disposes offsets, observer history and pending capture together.
            *self = Self {
                mount: Some((node.generation.clone(), kind)),
                ..Default::default()
            };
        }
        self.axes = node.style().overflow_axes();
        self.metrics.width = bounds.width;
        self.metrics.height = bounds.height;
        self.metrics.content_width = bounds.content_width.max(bounds.width);
        self.metrics.content_height = bounds.content_height.max(bounds.height);
        let mut target = self.metrics.offset;
        if let Some(request) = node.scroll_request
            && self.revision != Some(request.revision)
        {
            self.revision = Some(request.revision);
            target = request.offset;
        }
        self.set_offset(target);
    }
    /// Current clamped query, safe to copy across the application's bounded event channel.
    pub fn metrics(&self) -> ScrollMetrics {
        self.metrics
    }
    /// Clamp a requested position; hidden axes cannot be scrolled in this first contract.
    pub fn set_offset(&mut self, position: ScrollOffset) -> bool {
        if !position.x.is_finite() || !position.y.is_finite() {
            return false;
        }
        let next = ScrollOffset {
            x: if self.axes.0.scrollable() {
                position.x.clamp(
                    0.0,
                    (self.metrics.content_width - self.metrics.width).max(0.0),
                )
            } else {
                0.0
            },
            y: if self.axes.1.scrollable() {
                position.y.clamp(
                    0.0,
                    (self.metrics.content_height - self.metrics.height).max(0.0),
                )
            } else {
                0.0
            },
        };
        let changed = next != self.metrics.offset;
        self.metrics.offset = next;
        if changed {
            self.indicators.touch();
        }
        changed
    }
    /// Return true only if this viewport consumes the event; a blocked nested event may bubble.
    pub fn wheel(&mut self, mut x: f32, mut y: f32, shift: bool) -> bool {
        // Native macOS events may already map Shift to X. Preserve that axis and
        // the OS-adjusted sign; only remap a vertical-only wheel gesture.
        if shift && x == 0.0 {
            x = y;
            y = 0.0;
        }
        if self.axes.0.scrollable() && !self.axes.1.scrollable() && x == 0.0 {
            x = y;
            y = 0.0;
        }
        self.set_offset(ScrollOffset {
            x: self.metrics.offset.x + x,
            y: self.metrics.offset.y + y,
        })
    }
    /// Apply viewport keyboard commands without intercepting an editor's own keys.
    pub fn key(&mut self, key: ScrollKey) -> bool {
        let size = if self.axes.1.scrollable() {
            self.metrics.height
        } else {
            self.metrics.width
        };
        let page = (size - LINE_STEP).max(LINE_STEP);
        match key {
            ScrollKey::Left => self.wheel(-LINE_STEP, 0.0, false),
            ScrollKey::Right => self.wheel(LINE_STEP, 0.0, false),
            ScrollKey::Up => self.wheel(0.0, -LINE_STEP, false),
            ScrollKey::Down => self.wheel(0.0, LINE_STEP, false),
            ScrollKey::PageUp => self.wheel(0.0, -page, false),
            ScrollKey::PageDown => self.wheel(0.0, page, false),
            ScrollKey::Home => {
                let mut offset = self.metrics.offset;
                if self.axes.1.scrollable() {
                    offset.y = 0.0;
                } else {
                    offset.x = 0.0;
                }
                self.set_offset(offset)
            }
            ScrollKey::End => {
                let mut offset = self.metrics.offset;
                if self.axes.1.scrollable() {
                    offset.y = self.metrics.content_height;
                } else {
                    offset.x = self.metrics.content_width;
                }
                self.set_offset(offset)
            }
        }
    }
    /// Reveal content geometry, clamped to the available axes.
    pub fn reveal(&mut self, rect: ClipRect) -> bool {
        let axis = |start: f32, size: f32, offset: f32, viewport: f32| {
            if start < offset {
                start
            } else if start + size > offset + viewport {
                if size > viewport {
                    start
                } else {
                    start + size - viewport
                }
            } else {
                offset
            }
        };
        self.set_offset(ScrollOffset {
            x: axis(
                rect.x,
                rect.width,
                self.metrics.offset.x,
                self.metrics.width,
            ),
            y: axis(
                rect.y,
                rect.height,
                self.metrics.offset.y,
                self.metrics.height,
            ),
        })
    }
    /// Reveal on focus transitions, allowing deliberate wheel scrolling while an editor keeps focus.
    pub fn focus(&mut self, target: Option<(String, ClipRect)>) -> bool {
        let id = target.as_ref().map(|(id, _)| id.clone());
        let changed = self.focus != id;
        self.focus = id;
        changed && target.is_some_and(|(_, rect)| self.reveal(rect))
    }
    /// Notify only changed metrics; unchanged repaints cannot enqueue an update loop.
    pub fn notify(&mut self, node: &Node) -> Result<(), AdapterError> {
        if let Some(event) = &node.on_scroll
            && self.published != Some(self.metrics)
        {
            event
                .emit(self.metrics)
                .map_err(|error| AdapterError::Application(error.to_string()))?;
            self.published = Some(self.metrics);
        }
        Ok(())
    }
    /// Show auto indicators only during recent movement or a captured drag; scroll remains persistent.
    pub fn indicator_active(&self) -> bool {
        self.dragging()
            || self.axes.0 == Overflow::Scroll
            || self.axes.1 == Overflow::Scroll
            || self.indicators.remaining().is_some()
    }
    /// Native hosts stop scheduling when the activity window expires.
    pub fn indicator_delay(&self) -> Option<std::time::Duration> {
        self.indicators.remaining()
    }
    /// Claim one asynchronous expiry notification; repeated paints do not create more timers.
    pub fn take_indicator_timer(&mut self) -> Option<std::time::Duration> {
        self.indicators.schedule()
    }
    /// Release the timer claim after a native expiry wake-up.
    pub fn finish_indicator_timer(&mut self) {
        self.indicators.finished();
    }
    /// Derive identical overlay tracks/thumbs for all native painters.
    pub fn bars(&self) -> Vec<ScrollBar> {
        let mut bars = Vec::with_capacity(2);
        let visible = |axis: Overflow, content: f32, size: f32| {
            axis == Overflow::Scroll || (axis == Overflow::Auto && content > size)
        };
        let x = visible(self.axes.0, self.metrics.content_width, self.metrics.width);
        let y = visible(
            self.axes.1,
            self.metrics.content_height,
            self.metrics.height,
        );
        for horizontal in [false, true] {
            if !(if horizontal { x } else { y }) {
                continue;
            }
            let (size, content, offset, length) = if horizontal {
                (
                    self.metrics.width,
                    self.metrics.content_width,
                    self.metrics.offset.x,
                    (self.metrics.width - if y { BAR_SIZE } else { 0.0 }).max(0.0),
                )
            } else {
                (
                    self.metrics.height,
                    self.metrics.content_height,
                    self.metrics.offset.y,
                    (self.metrics.height - if x { BAR_SIZE } else { 0.0 }).max(0.0),
                )
            };
            let thumb = (length * size / content.max(1.0)).max(20.0).min(length);
            let position = if content > size {
                (length - thumb) * offset / (content - size)
            } else {
                0.0
            };
            let track = if horizontal {
                ClipRect {
                    x: 0.0,
                    y: (self.metrics.height - BAR_SIZE).max(0.0),
                    width: length,
                    height: BAR_SIZE.min(self.metrics.height),
                }
            } else {
                ClipRect {
                    x: (self.metrics.width - BAR_SIZE).max(0.0),
                    y: 0.0,
                    width: BAR_SIZE.min(self.metrics.width),
                    height: length,
                }
            };
            if track.width <= 0.0 || track.height <= 0.0 {
                continue;
            }
            let thumb = if horizontal {
                ClipRect {
                    x: position,
                    width: thumb,
                    ..track
                }
            } else {
                ClipRect {
                    y: position,
                    height: thumb,
                    ..track
                }
            };
            bars.push(ScrollBar {
                horizontal,
                track,
                thumb,
            });
        }
        bars
    }
    /// Start thumb dragging or jump the track under a pointer.
    pub fn press(&mut self, x: f32, y: f32) -> bool {
        for bar in self.bars() {
            if bar.track.contains(x, y) {
                let pos = if bar.horizontal { x } else { y };
                let start = if bar.horizontal {
                    bar.thumb.x
                } else {
                    bar.thumb.y
                };
                let length = if bar.horizontal {
                    bar.thumb.width
                } else {
                    bar.thumb.height
                };
                self.drag = Some((
                    bar.horizontal,
                    if bar.thumb.contains(x, y) {
                        pos - start
                    } else {
                        length / 2.0
                    },
                ));
                self.drag_to(x, y);
                return true;
            }
        }
        false
    }
    /// Continue a captured thumb drag, even beyond the viewport.
    pub fn drag_to(&mut self, x: f32, y: f32) -> bool {
        let Some((horizontal, grab)) = self.drag else {
            return false;
        };
        let Some(bar) = self
            .bars()
            .into_iter()
            .find(|bar| bar.horizontal == horizontal)
        else {
            return false;
        };
        let (pointer, track, thumb, max) = if horizontal {
            (
                x,
                bar.track.width,
                bar.thumb.width,
                self.metrics.content_width - self.metrics.width,
            )
        } else {
            (
                y,
                bar.track.height,
                bar.thumb.height,
                self.metrics.content_height - self.metrics.height,
            )
        };
        let target = if track > thumb {
            (pointer - grab) / (track - thumb) * max
        } else {
            0.0
        };
        let mut offset = self.metrics.offset;
        if horizontal {
            offset.x = target;
        } else {
            offset.y = target;
        }
        self.set_offset(offset)
    }
    /// End native pointer capture.
    pub fn release(&mut self) {
        self.drag = None;
    }
    /// Whether the binding must keep pointer capture for a thumb.
    pub fn dragging(&self) -> bool {
        self.drag.is_some()
    }
}
/// Non-visible overflow establishes a paint/input viewport.
pub fn clips(style: &Style) -> bool {
    style.overflow_axes() != (Overflow::Visible, Overflow::Visible)
}
/// Whether a viewport participates in wheel/keyboard scrolling.
pub fn scrollable(style: &Style) -> bool {
    let (x, y) = style.overflow_axes();
    x.scrollable() || y.scrollable()
}
