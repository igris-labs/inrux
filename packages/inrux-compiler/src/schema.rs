//! Built-in authoring names shared by validation and editor completion metadata.
pub(crate) const BUILTINS: &[&str] = &[
    "Markdown",
    "Flex",
    "ScrollArea",
    "Card",
    "Badge",
    "Separator",
    "Grid",
    "AspectRatio",
    "Box",
    "Container",
    "Center",
    "Spacer",
    "Column",
    "Row",
    "Text",
    "Button",
    "Input",
    "TextInput",
    "TextArea",
    "If",
    "Heartbeat",
    "Component",
    "Switch",
    "Progress",
    "Meter",
    "Checkbox",
    "Select",
    "Dialog",
    "Slot",
    "Children",
    "Provide",
    "For",
    "ElseIf",
    "Else",
];

/// Keep common styling attributes available on both built-in and imported roots.
pub(crate) fn attributes(tag: &str) -> Vec<&'static str> {
    let mut accepted = vec![
        "id", "class", "style", "gap", "padding", "width", "height", "key",
    ];
    accepted.extend(match tag {
        "Markdown" => vec!["document"],
        "Flex" => vec!["direction", "wrap"],
        "Grid" => vec!["columns"],
        "AspectRatio" => vec!["ratio"],
        "Text" | "Badge" => vec!["text"],
        "Separator" => vec!["vertical"],
        "ScrollArea" => vec!["scroll_to", "@scroll"],
        "Button" => vec!["text", "disabled", "@click"],
        "Input" | "TextInput" | "TextArea" => {
            vec!["value", "model:value", "@input", "placeholder", "disabled"]
        }
        "If" | "ElseIf" => vec!["condition"],
        "Progress" => vec!["label", "value", "max"],
        "Meter" => vec!["label", "value", "min", "max"],
        "Checkbox" | "Switch" => {
            vec!["checked", "model:checked", "@change", "label", "disabled"]
        }
        "Select" => vec!["value", "model:value", "options", "@change", "disabled"],
        "Dialog" => vec!["open", "@close"],
        "Provide" => vec!["value"],
        "For" => vec!["each", "item"],
        "Component" => vec!["is"],
        _ => vec![],
    });
    accepted
}
