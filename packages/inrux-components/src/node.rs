//! Owned, backend-independent UI descriptions.
use inrux_core::Emit;
use inrux_style::{CompiledStyles, Part, Rule, State as StyleState, Style, Viewport};
use std::sync::Arc;

/// The primitive represented by a render node.
#[derive(Clone)]
pub enum NodeKind {
    /// Vertical layout.
    Column,
    /// Horizontal layout.
    Row,
    /// Owned text.
    Text(String),
    /// Button label and activation event.
    Button(String, Option<Emit<()>>),
    /// Controlled text input.
    Input {
        /// Whether hard line breaks and wrapped multiline editing are enabled.
        multiline: bool,
        /// Controlled model value.
        value: String,
        /// Empty-value hint.
        placeholder: String,
        /// Queued edit event.
        event: crate::InputEvent,
    },
    /// Labelled boolean control.
    Checkbox {
        /// Semantic control kind, independent of class/style names.
        kind: crate::ToggleKind,
        /// Visible accessible label.
        label: String,
        /// Controlled selection state.
        checked: bool,
        /// Typed value change event.
        event: Emit<bool>,
    },
    /// Controlled determinate progress or bounded measurement.
    Range(crate::RangeValue),
    /// Single-choice dropdown.
    Select {
        /// Controlled selected value.
        value: String,
        /// Available display values.
        options: Vec<String>,
        /// Typed selection event.
        event: Emit<String>,
    },
    /// Controlled modal that keeps its content outside the normal layout flow.
    Dialog {
        /// Whether the modal is visible.
        open: bool,
        /// Escape or backdrop close request.
        on_close: Emit<()>,
    },
    /// UI-thread animation diagnostic.
    Heartbeat,
    /// Excluded node.
    Empty,
}
/// Renderer-independent component output. Event callbacks only enqueue worker commands.
#[derive(Clone)]
pub struct Node {
    /// Optional UI-owned viewport metrics observer.
    pub on_scroll: Option<Emit<crate::ScrollMetrics>>,
    /// Optional revision-bearing viewport command.
    pub scroll_request: Option<crate::ScrollRequest>,
    /// Primitive handled by the native adapter.
    pub kind: NodeKind,
    /// Ordered child render descriptions.
    pub children: Vec<Node>,
    /// Component defaults below global, scoped and explicit styles.
    pub defaults: Style,
    /// Resolved global and scoped class base rules.
    pub base: Style,
    /// Explicit Rust and template layout overrides.
    pub inline: Style,
    /// Interaction paint layers in stylesheet source order.
    pub states: Vec<(StyleState, Style)>,
    /// Compiled owner-qualified public-part rules in cascade order.
    pub part_rules: Vec<(Part, StyleState, Style)>,
    /// Explicit Rust part overrides, above compiled rules.
    pub part_inline: Vec<(Part, Style)>,
    /// Whether the native control accepts user interaction.
    pub disabled: bool,
    /// Authored class names, retained for application-global matching.
    pub classes: Vec<String>,
    /// Case-sensitive framework component type used by compiled selectors.
    pub component_type: &'static str,
    /// Style ID; independent of native retention identity.
    pub id: Option<String>,
    /// Shared, pre-indexed component-local stylesheets.
    pub scoped_styles: Vec<Arc<CompiledStyles>>,
    /// Stable sibling identity used when lists reorder.
    pub key: Option<String>,
    /// Mount generation prevents remounted controls from inheriting disposed native editing state.
    pub generation: Option<String>,
    /// Retained mounts contributing this node; used to dispose eagerly composed but unused slots.
    pub mounts: Vec<String>,
}
impl Node {
    /// Construct one owned render descriptor.
    pub(crate) fn new(kind: NodeKind, children: Vec<Self>) -> Self {
        Self {
            on_scroll: None,
            scroll_request: None,
            component_type: match &kind {
                NodeKind::Column => "Column",
                NodeKind::Row => "Row",
                NodeKind::Text(_) => "Text",
                NodeKind::Button(..) => "Button",
                NodeKind::Input {
                    multiline: true, ..
                } => "TextArea",
                NodeKind::Input { .. } => "Input",
                NodeKind::Checkbox {
                    kind: crate::ToggleKind::Switch,
                    ..
                } => "Switch",
                NodeKind::Checkbox { .. } => "Checkbox",
                NodeKind::Range(range) => match range.kind {
                    crate::RangeKind::Progress => "Progress",
                    crate::RangeKind::Meter => "Meter",
                },
                NodeKind::Select { .. } => "Select",
                NodeKind::Dialog { .. } => "Dialog",
                NodeKind::Heartbeat => "Heartbeat",
                NodeKind::Empty => "Empty",
            },
            id: None,
            scoped_styles: vec![],
            kind,
            children,
            defaults: Style::new(),
            base: Style::new(),
            inline: Style::new(),
            states: vec![],
            part_rules: vec![],
            part_inline: vec![],
            disabled: false,
            classes: vec![],
            key: None,
            generation: None,
            mounts: vec![],
        }
    }
    /// Set the disabled state of an interactive node.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
    /// Supply reusable component defaults below global/scoped/explicit styling.
    pub fn with_defaults(mut self, style: Style) -> Self {
        self.defaults.overlay(&style);
        self
    }
    /// Apply a Rust style above class rules, replacing only set properties.
    pub fn styled(mut self, style: Style) -> Self {
        self.inline.overlay(&style);
        self
    }
    /// Add authored classes without parsing CSS. Repeated classes are deduplicated.
    pub fn classes(mut self, names: &str) -> Self {
        for name in names.split_whitespace() {
            if !self.classes.iter().any(|existing| existing == name) {
                self.classes.push(name.into());
            }
        }
        self
    }
    /// Compatibility helper for runtime rules. Reuse with_styles for hot construction paths.
    pub fn class(self, names: &str, rules: &[Rule]) -> Self {
        let node = self.classes(names);
        if rules.is_empty() {
            node
        } else {
            let mut node = node.with_styles(Arc::new(CompiledStyles::new(rules.to_vec())));
            node.resolve_styles(&[], Viewport::default());
            node
        }
    }
    /// Attach an immutable indexed local sheet; media is re-resolved by the coordinator.
    pub fn with_styles(mut self, sheet: Arc<CompiledStyles>) -> Self {
        if !sheet.rules().is_empty() {
            self.scoped_styles.push(sheet);
        }
        self
    }
    /// Set a style ID without changing retained widget identity.
    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }
    /// Preserve the authored component type through shared primitive constructors.
    pub fn component_type(mut self, name: &'static str) -> Self {
        self.component_type = name;
        self
    }
    /// Preserve widget identity across sibling reordering; keys must be unique within a parent.
    pub fn key(mut self, key: impl ToString) -> Self {
        self.key = Some(key.to_string());
        self
    }
    /// Compatibility path for a one-off global rule list; sessions reuse indexed sheets instead.
    pub fn apply_globals(&mut self, rules: &[Rule]) {
        self.resolve_styles(&[&CompiledStyles::new(rules.to_vec())], Viewport::default());
    }
    /// Resolve globals followed by scoped sheets, then recurse; inline styles remain highest priority.
    pub fn resolve_styles(&mut self, globals: &[&CompiledStyles], viewport: Viewport) {
        let mut base = Style::new();
        let mut layers = Vec::new();
        let mut parts = Vec::new();
        for sheet in globals
            .iter()
            .copied()
            .chain(self.scoped_styles.iter().map(Arc::as_ref))
        {
            for rule in sheet.matching(
                self.component_type,
                self.id.as_deref(),
                &self.classes,
                viewport,
            ) {
                if let Some(part) = rule.selector.part {
                    parts.push((part, rule.state, rule.style.clone()));
                    continue;
                }
                if rule.state == StyleState::Base {
                    base.overlay(&rule.style);
                    if !layers.is_empty() {
                        layers.push((rule.state, rule.style.clone()));
                    }
                } else {
                    if layers.is_empty() {
                        layers.push((StyleState::Base, base.clone()));
                    }
                    layers.push((rule.state, rule.style.clone()));
                }
            }
        }
        self.base = base;
        self.states = layers;
        self.part_rules = parts;
        for child in &mut self.children {
            child.resolve_styles(globals, viewport);
        }
    }
    /// Reject duplicate sibling keys before they alias native editing state.
    pub fn validate_keys(&self) -> Result<(), String> {
        let mut seen = std::collections::HashSet::new();
        for child in &self.children {
            if let Some(key) = &child.key
                && !seen.insert(key)
            {
                return Err(format!("Duplicate sibling key `{key}`"));
            }
            child.validate_keys()?;
        }
        Ok(())
    }
    /// Resolve base styles and explicit overrides.
    pub fn style(&self) -> Style {
        let mut result = self.defaults.clone();
        result.overlay(&self.base);
        result.overlay(&self.inline);
        result
    }
    /// Apply inherited text properties before publishing a render tree.
    pub fn inherit(&mut self, parent: &Style) {
        let own = self.style();
        if own.accent_color.is_none() {
            self.base.accent_color = parent.accent_color;
        }
        if own.white_space.is_none() {
            self.base.white_space = parent.white_space;
        }
        if own.scrollbar_colors.is_none() {
            self.base.scrollbar_colors = parent.scrollbar_colors;
        }
        if own.color.is_none() {
            self.base.color = parent.color;
        }
        if own.font_family.is_none() {
            self.base.font_family.clone_from(&parent.font_family);
        }
        if own.font_size.is_none() {
            self.base.font_size = parent.font_size;
        }
        let style = self.style();
        for child in &mut self.children {
            child.inherit(&style);
        }
    }
}
