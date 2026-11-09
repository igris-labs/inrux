//! Owner-state evaluation and stable component-part styling, without native widget handles.
use crate::{Node, NodeKind, ToggleKind};
use inrux_style::{Part, State, Style};
impl Node {
    /// Authored focus borders replace the fallback ring instead of receiving an extra native outline.
    pub fn has_focus_decoration(&self) -> bool {
        self.states.iter().any(|(state, style)| {
            *state == State::Focus && (style.border_color.is_some() || style.border_width.is_some())
        }) || self.part_rules.iter().any(|(part, state, style)| {
            matches!(part, Part::Track | Part::Control)
                && *state == State::Focus
                && (style.border_color.is_some() || style.border_width.is_some())
        })
    }

    /// Native bindings only need hover observation when it can affect authored paint.
    pub fn has_hover_styles(&self) -> bool {
        self.states.iter().any(|(state, _)| *state == State::Hover)
            || self
                .part_rules
                .iter()
                .any(|(_, state, _)| *state == State::Hover)
    }

    /// Retain owner-only paint data in native bindings without cloning descendant subtrees or sheets.
    pub fn paint_snapshot(&self) -> Self {
        let mut owner = Self::new(self.kind.clone(), vec![]);
        owner.component_type = self.component_type;
        owner.disabled = self.disabled;
        owner.defaults = self.defaults.clone();
        owner.base = self.base.clone();
        owner.inline = self.inline.clone();
        owner.states = self.states.clone();
        owner.part_rules = self.part_rules.clone();
        owner.part_inline = self.part_inline.clone();
        owner
    }

    /// Override a public visual part above global/scoped rules without changing its behavior.
    pub fn styled_part(mut self, part: Part, style: Style) -> Self {
        if let Some((_, current)) = self
            .part_inline
            .iter_mut()
            .find(|(target, _)| *target == part)
        {
            current.overlay(&style);
        } else {
            self.part_inline.push((part, style));
        }
        self
    }
    /// Test an owner's semantic/native state; hover is suppressed while disabled.
    pub fn matches_style_state(&self, state: State, hovered: bool, focused: bool) -> bool {
        match state {
            State::Base => true,
            State::Hover => hovered && !self.disabled,
            State::Focus => focused && !self.disabled,
            State::Disabled => self.disabled,
            State::Checked => matches!(self.kind, NodeKind::Checkbox { checked: true, .. }),
            State::Unchecked => matches!(self.kind, NodeKind::Checkbox { checked: false, .. }),
        }
    }
    /// Resolve owner paint at native event time while preserving explicit-style precedence.
    pub fn paint_style(&self, hovered: bool, focused: bool) -> Style {
        let mut result = self.defaults.clone();
        if self.states.is_empty() {
            result.overlay(&self.base);
        } else {
            for (state, style) in &self.states {
                if self.matches_style_state(*state, hovered, focused) {
                    result.overlay(style);
                }
            }
        }
        result.overlay(&self.inline);
        result
    }
    /// Resolve only authored part properties; native defaults and inherited label ink stay separate.
    pub fn part_style(&self, part: Part, hovered: bool, focused: bool) -> Style {
        let mut result = Style::new();
        for (target, state, style) in &self.part_rules {
            if *target == part && self.matches_style_state(*state, hovered, focused) {
                result.overlay(style);
            }
        }
        if let Some((_, style)) = self.part_inline.iter().find(|(target, _)| *target == part) {
            result.overlay(style);
        }
        result
    }
    /// Report the actual component's public parts rather than accepting ignored native selectors.
    pub fn supports_part(&self, part: Part) -> bool {
        match self.kind {
            NodeKind::Checkbox {
                kind: ToggleKind::Switch,
                ..
            } => matches!(part, Part::Track | Part::Thumb | Part::Label),
            NodeKind::Checkbox { .. } => {
                matches!(part, Part::Control | Part::Indicator | Part::Label)
            }
            NodeKind::Input { .. } => part == Part::Placeholder,
            NodeKind::Range(_) => part == Part::Indicator,
            _ => false,
        }
    }
}
