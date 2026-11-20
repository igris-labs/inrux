// Copyright 2021 the Parley Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Adapted from Parley 0.6.0's AccessKit bridge for the current Parley layout API.

//! Adapter-owned accessible text runs and selection, independent of Parley's removed integration.
use masonry::accesskit::{self, Node, NodeId, Role, TextDirection, TreeUpdate};
use parley::{Brush, Layout};
use std::collections::{HashMap, HashSet};

#[derive(Clone, Default)]
pub(crate) struct LayoutAccessibility {
    // The following two fields maintain a two-way mapping between runs
    // and AccessKit node IDs, where each run is identified by its line index
    // and run index within that line, or a run path for short. These maps
    // are maintained by `LayoutAccess::build_nodes`, which ensures that removed
    // runs are removed from the maps on the next accessibility pass.
    pub(crate) access_ids_by_run_path: HashMap<(usize, usize), NodeId>,
    pub(crate) run_paths_by_access_id: HashMap<NodeId, (usize, usize)>,
    byte_boundaries: HashMap<NodeId, Vec<usize>>,
}

impl LayoutAccessibility {
    #[allow(clippy::too_many_arguments)]
    pub fn build_nodes<B: Brush>(
        &mut self,
        text: &str,
        layout: &Layout<B>,
        update: &mut TreeUpdate,
        parent_node: &mut Node,
        mut next_node_id: impl FnMut() -> NodeId,
        x_offset: f64,
        y_offset: f64,
    ) {
        // Build a set of node IDs for the runs encountered in this pass.
        let mut ids = HashSet::<NodeId>::new();
        // Reuse scratch space for storing a sorted list of runs.
        let mut runs = Vec::new();

        for (line_index, line) in layout.lines().enumerate() {
            let metrics = line.metrics();
            // Defer adding each run node until we reach either the next run
            // or the end of the line. That way, we can set relations between
            // runs in a line and do anything special that might be required
            // for the last run in a line.
            let mut last_node: Option<(NodeId, Node)> = None;

            // Iterate over the runs from left to right, computing their offsets,
            // then sort them into text order.
            runs.clear();
            runs.reserve(line.len());
            {
                let mut run_offset = metrics.offset;
                for run in line.runs() {
                    let advance = run.advance();
                    runs.push((run, run_offset));
                    run_offset += advance;
                }
            }
            runs.sort_by_key(|(r, _)| r.text_range().start);

            for (run, run_offset) in runs.drain(..) {
                let run_path = (line_index, run.index());
                // If we encountered this same run path in the previous
                // accessibility pass, reuse the same AccessKit ID. Otherwise,
                // allocate a new one. This enables stable node IDs when merely
                // updating the content of existing runs.
                let id = self
                    .access_ids_by_run_path
                    .get(&run_path)
                    .copied()
                    .unwrap_or_else(|| {
                        let id = next_node_id();
                        self.access_ids_by_run_path.insert(run_path, id);
                        self.run_paths_by_access_id.insert(id, run_path);
                        id
                    });
                ids.insert(id);
                let mut node = Node::new(Role::TextRun);

                if let Some((last_id, mut last_node)) = last_node.take() {
                    last_node.set_next_on_line(id);
                    node.set_previous_on_line(last_id);
                    update.nodes.push((last_id, last_node));
                    parent_node.push_child(last_id);
                }

                node.set_bounds(accesskit::Rect {
                    x0: x_offset + run_offset as f64,
                    y0: y_offset + metrics.block_min_coord as f64,
                    x1: x_offset + (run_offset + run.advance()) as f64,
                    y1: y_offset + metrics.block_max_coord as f64,
                });
                node.set_text_direction(if run.is_rtl() {
                    TextDirection::RightToLeft
                } else {
                    TextDirection::LeftToRight
                });

                let run_text = &text[run.text_range()];
                node.set_value(run_text);

                let mut boundaries = Vec::new();
                let mut character_lengths = Vec::new();
                let mut cluster_offset = 0.0;
                let mut character_positions = Vec::new();
                let mut character_widths = Vec::new();
                let mut word_lengths = Vec::new();
                let mut last_word_start = 0;

                for cluster in run.clusters() {
                    boundaries.push(cluster.text_range().start);
                    let cluster_text = &text[cluster.text_range()];
                    if cluster.is_word_boundary()
                        && !cluster.is_space_or_nbsp()
                        && !character_lengths.is_empty()
                    {
                        word_lengths.push((character_lengths.len() - last_word_start) as _);
                        last_word_start = character_lengths.len();
                    }
                    character_lengths.push(cluster_text.len() as _);
                    character_positions.push(cluster_offset);
                    character_widths.push(cluster.advance());
                    cluster_offset += cluster.advance();
                }

                boundaries.push(run.text_range().end);
                self.byte_boundaries.insert(id, boundaries);
                word_lengths.push((character_lengths.len() - last_word_start) as _);
                node.set_character_lengths(character_lengths);
                node.set_character_positions(character_positions);
                node.set_character_widths(character_widths);
                node.set_word_lengths(word_lengths);

                last_node = Some((id, node));
            }

            if let Some((id, node)) = last_node {
                update.nodes.push((id, node));
                parent_node.push_child(id);
            }
        }

        // Remove mappings for runs that no longer exist.
        let mut ids_to_remove = Vec::<NodeId>::new();
        let mut run_paths_to_remove = Vec::<(usize, usize)>::new();
        for (access_id, run_path) in self.run_paths_by_access_id.iter() {
            if !ids.contains(access_id) {
                ids_to_remove.push(*access_id);
                run_paths_to_remove.push(*run_path);
            }
        }
        for id in ids_to_remove {
            self.run_paths_by_access_id.remove(&id);
            self.byte_boundaries.remove(&id);
        }
        for run_path in run_paths_to_remove {
            self.access_ids_by_run_path.remove(&run_path);
        }
    }
}

impl LayoutAccessibility {
    /// Resolve a semantic position to a valid UTF-8 cluster boundary in the current layout.
    pub(crate) fn byte_index(&self, position: &accesskit::TextPosition) -> Option<usize> {
        self.byte_boundaries
            .get(&position.node)?
            .get(position.character_index)
            .copied()
    }
    /// Publish selection in the same run IDs used for the accessible text children.
    pub(crate) fn selection(
        &self,
        anchor: usize,
        focus: usize,
    ) -> Option<accesskit::TextSelection> {
        Some(accesskit::TextSelection {
            anchor: self.position(anchor)?,
            focus: self.position(focus)?,
        })
    }
    fn position(&self, byte: usize) -> Option<accesskit::TextPosition> {
        // Prefer the next run at a shared boundary, independent of hash iteration order.
        let (&node, boundaries) = self
            .byte_boundaries
            .iter()
            .filter(|(_, indices)| {
                indices.first().is_some_and(|start| *start <= byte)
                    && indices.last().is_some_and(|end| byte <= *end)
            })
            .max_by_key(|(_, indices)| indices[0])?;
        let character_index = boundaries.iter().position(|index| *index == byte)?;
        Some(accesskit::TextPosition {
            node,
            character_index,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use masonry::core::WidgetId;

    fn build(access: &mut LayoutAccessibility, text: &str) -> (Node, TreeUpdate) {
        let shaped = inrux_text::shape(text, 15.0, None, 400.0);
        let mut parent = Node::new(Role::TextInput);
        let mut update = TreeUpdate {
            nodes: vec![],
            tree: None,
            focus: NodeId(0),
        };
        access.build_nodes(
            text,
            &shaped.layout,
            &mut update,
            &mut parent,
            || NodeId::from(WidgetId::next()),
            -10.0,
            -5.0,
        );
        (parent, update)
    }

    #[test]
    fn unicode_selection_round_trips_through_accessible_runs() {
        let mut access = LayoutAccessibility::default();
        let text = "café 👩‍💻\nשלום";
        let (parent, update) = build(&mut access, text);
        assert!(!parent.children().is_empty());
        assert!(
            update
                .nodes
                .iter()
                .all(|(_, node)| node.role() == Role::TextRun)
        );
        let selection = access.selection("café ".len(), text.len()).unwrap();
        assert_eq!(access.byte_index(&selection.anchor), Some("café ".len()));
        assert_eq!(access.byte_index(&selection.focus), Some(text.len()));
        assert!(
            access.selection(4, text.len()).is_none(),
            "UTF-8 continuation bytes cannot become selections"
        );
    }

    #[test]
    fn rebuilt_runs_keep_ids_and_remove_stale_selection_targets() {
        let mut access = LayoutAccessibility::default();
        let (parent, _) = build(&mut access, "first\nsecond");
        let first = parent.children()[0];
        let old = access.selection(0, "first\nsecond".len()).unwrap();
        let (parent, _) = build(&mut access, "first");
        assert_eq!(parent.children()[0], first);
        assert_eq!(access.byte_index(&old.focus), None);
        assert_eq!(
            access.byte_index(&access.selection(0, 5).unwrap().focus),
            Some(5)
        );
    }
}
