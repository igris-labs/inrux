//! One catalogue defines sidebar labels, headings and usage text.
/// Portable specimen selected by the application, not by an engine adapter.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Example {
    Button,
    Input,
    TextArea,
    Checkbox,
    Switch,
    Progress,
    Meter,
    Text,
    Badge,
    Card,
    Separator,
    Box,
    Container,
    Row,
    Column,
    Center,
    Spacer,
    Flex,
    Grid,
    AspectRatio,
    ScrollArea,
}
/// Navigation order; each entry has a real working preview.
pub const EXAMPLES: &[Example] = &[
    Example::Button,
    Example::Input,
    Example::TextArea,
    Example::Checkbox,
    Example::Switch,
    Example::Progress,
    Example::Meter,
    Example::Text,
    Example::Badge,
    Example::Card,
    Example::Separator,
    Example::Box,
    Example::Container,
    Example::Row,
    Example::Column,
    Example::Center,
    Example::Spacer,
    Example::Flex,
    Example::Grid,
    Example::AspectRatio,
    Example::ScrollArea,
];
impl Example {
    /// Display the same component spelling in navigation, header and snippets.
    pub fn name(self) -> &'static str {
        match self {
            Self::Button => "Button",
            Self::Input => "Input",
            Self::TextArea => "TextArea",
            Self::Checkbox => "Checkbox",
            Self::Switch => "Switch",
            Self::Progress => "Progress",
            Self::Meter => "Meter",
            Self::Text => "Text",
            Self::Badge => "Badge",
            Self::Card => "Card",
            Self::Separator => "Separator",
            Self::Box => "Box",
            Self::Container => "Container",
            Self::Row => "Row",
            Self::Column => "Column",
            Self::Center => "Center",
            Self::Spacer => "Spacer",
            Self::Flex => "Flex",
            Self::Grid => "Grid",
            Self::AspectRatio => "AspectRatio",
            Self::ScrollArea => "ScrollArea",
        }
    }
    /// Explain this component's implemented contract without promising deferred features.
    pub fn description(self) -> &'static str {
        match self {
            Self::Button => "Actions, disabled admission and typed click events.",
            Self::Input => "A controlled single-line value, including pasted text.",
            Self::TextArea => "Multiple lines, native wrapping and caret-aware scrolling.",
            Self::Checkbox => "A boolean model with ordinary keyboard and pointer interaction.",
            Self::Switch => {
                "A controlled on/off setting with native pointer, keyboard and disabled behavior."
            }
            Self::Progress => "Determinate completion with explicit bounds and theme-aware fill.",
            Self::Meter => "A bounded read-only measurement, distinct from completion or a slider.",
            Self::Text => "Inherited typography and values updated from Rust state.",
            Self::Badge => "A compact text label with overridable colors and spacing.",
            Self::Card => "A reusable surface that keeps child events and models intact.",
            Self::Separator => "Decorative horizontal and vertical dividers.",
            Self::Box => "A general-purpose physical container with child layout.",
            Self::Container => "A width-constrained container centered in its parent.",
            Self::Row => "A horizontal layout with predictable spacing between children.",
            Self::Column => "A vertical layout with explicit gaps and ordinary children.",
            Self::Center => "Center content across both axes inside bounded geometry.",
            Self::Spacer => "Share remaining space between siblings with flex growth.",
            Self::Flex => "Change layout direction while retaining child identity.",
            Self::Grid => "Equal-width tracks with automatic rows.",
            Self::AspectRatio => "Keep a preferred width-to-height ratio inside bounded space.",
            Self::ScrollArea => "A clipped viewport with native input and shared scroll policy.",
        }
    }
    /// A minimal .iru usage example corresponding to the live specimen.
    pub fn usage(self) -> &'static str {
        match self {
            Self::Button => r##"<Button text="Add to count" @click={increment} />"##,
            Self::Input => r##"<Input model:value={state.value} placeholder="Workspace name" />"##,
            Self::TextArea => {
                r##"<TextArea model:value={state.notes} height={Size::Px(160.0)} />"##
            }
            Self::Checkbox => {
                r##"<Checkbox label="Enable notifications" model:checked={state.checked} />"##
            }
            Self::Switch => r##"<Switch label="Automatic sync" model:checked={state.checked} />"##,
            Self::Progress => r##"<Progress label="Build progress" value={25.0} max={100.0} />"##,
            Self::Meter => r##"<Meter label="Temperature" value={20.0} min={-10.0} max={40.0} />"##,
            Self::Text => r##"<Text :text={format!("Count: {}", state.count)} />"##,
            Self::Badge => r##"<Badge text="Ready" class="saved-badge" />"##,
            Self::Card => {
                r##"<Card><Input model:value={state.value} /><Button text="Save" @click={increment} /></Card>"##
            }
            Self::Separator => r##"<Separator />"##,
            Self::Box => {
                r##"<Box padding={16.0} gap={12.0}><Text text="Contained content" /></Box>"##
            }
            Self::Container => {
                r##"<Container :style={Style::new().max_width(Size::Px(420.0))}><Text text="Centered content" /></Container>"##
            }
            Self::Row => r##"<Row gap={12.0}><Badge text="Alpha" /><Badge text="Beta" /></Row>"##,
            Self::Column => {
                r##"<Column gap={12.0}><Text text="First" /><Text text="Second" /></Column>"##
            }
            Self::Center => {
                r##"<Center height={Size::Px(160.0)}><Badge text="Centered" /></Center>"##
            }
            Self::Spacer => r##"<Row><Text text="Start" /><Spacer /><Text text="End" /></Row>"##,
            Self::Flex => {
                r##"<Flex :direction={FlexDirection::Row} gap={12.0}><Badge text="One" /><Badge text="Two" /></Flex>"##
            }
            Self::Grid => {
                r##"<Grid columns={3} gap={12.0}><Badge text="One" /><Badge text="Two" /><Badge text="Three" /></Grid>"##
            }
            Self::AspectRatio => {
                r##"<AspectRatio ratio={16.0 / 9.0} width={Size::Px(240.0)}><Center><Text text="16 : 9" /></Center></AspectRatio>"##
            }
            Self::ScrollArea => {
                r##"<ScrollArea height={Size::Px(180.0)}><For each={1..=12} item="row" key={row}><Text :text={format!("Row {}", row)} /></For></ScrollArea>"##
            }
        }
    }
}
