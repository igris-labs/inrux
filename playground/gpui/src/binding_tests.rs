//! Exercise shorthand through real generated Rust and headless event delivery.
const VIEW: &str = r#"
<Column>
    <Text text={count} />
    <Text text={doubled} />
    <Text text={counter.count} />
    <Input model:value={name} />
    <Input model:value={counter.name} />
    <Checkbox label="Enabled" model:checked={enabled} />
    <Child title={name} count={count} />
    <NameInput model:value={counter.name} id="custom-name" />
    <For each={rows} item="count" key={count}>
        <Text text={count} />
        <Text text={format!("row {}", count)} />
    </For>
    <Text text={{ let count = 99; format!("block {}", count) }} />
    <Text text={format!("value {}", count.get())} />
    <Text text={rows.iter().map(|count| count + 1).sum::<u32>()} />
</Column>
"#;
const SUPPORT: &str = r#"
use inrux::prelude::*;
use self::child as Child;
use self::name_input as NameInput;
#[derive(Clone)]
pub struct Counter { count: Signal<u32>, name: Signal<String> }
fn use_counter() -> Counter { Counter { count: Signal::new(7), name: Signal::new("Nested".into()) } }
"#;
const CHECKS: &str = r#"
#[cfg(test)] mod checks {
    use super::*;
    use inrux::{ComponentHost, NodeKind, Worker};
    fn text(node: &Node, expected: &str) -> bool {
        matches!(&node.kind, NodeKind::Text(value) if value == expected) || node.children.iter().any(|child| text(child, expected))
    }
    fn input(node: &Node, expected: &str) -> inrux::InputEvent {
        if let NodeKind::Input { value, event, .. } = &node.kind {
            if value == expected { return event.clone(); }
        }
        for child in &node.children {
            if contains_input(child, expected) { return input(child, expected); }
        }
        panic!("missing input {expected}")
    }
    fn named<'a>(node: &'a Node, id: &str) -> &'a Node {
        if node.id.as_deref() == Some(id) { return node; }
        node.children.iter().find_map(|child| find_named(child, id)).unwrap()
    }
    fn find_named<'a>(node: &'a Node, id: &str) -> Option<&'a Node> {
        if node.id.as_deref() == Some(id) { Some(node) } else { node.children.iter().find_map(|child| find_named(child, id)) }
    }
    fn contains_input(node: &Node, expected: &str) -> bool {
        matches!(&node.kind, NodeKind::Input { value, .. } if value == expected) || node.children.iter().any(|child| contains_input(child, expected))
    }
    #[test]
    fn busy_computed_reads_render_a_diagnostic_without_panicking() {
        let mut state = setup();
        let (started, ready) = std::sync::mpsc::channel();
        let (release, finish) = std::sync::mpsc::channel();
        let finish = std::sync::Mutex::new(finish);
        state.doubled = Computed::new(move || {
            started.send(()).unwrap();
            finish.lock().unwrap().recv().unwrap();
            1
        });
        let source = state.doubled.clone();
        let getter = std::thread::spawn(move || source.get());
        ready.recv_timeout(std::time::Duration::from_secs(2)).unwrap();
        let worker = Worker::spawn(state.clone(), 16, || {}).unwrap();
        let events = Events::new(worker.sender());
        let root = ComponentHost::default().render(&state, &events, &Scope::default());
        release.send(()).unwrap();
        getter.join().unwrap().unwrap();
        assert!(text(&root, "Cannot read binding `doubled`: Computed value is being evaluated on another thread"));
    }
    #[test]
    fn reads_are_reactive_and_models_keep_their_nested_targets() {
        let state = setup();
        let worker = Worker::spawn(state.clone(), 16, || {}).unwrap();
        let events = Events::new(worker.sender());
        let mut host = ComponentHost::default();
        let scope = Scope::default();
        let root = host.render(&state, &events, &scope);
        for expected in ["2", "4", "7", "row 3", "block 99", "value 2", "child Root 2", "8"] { assert!(text(&root, expected), "missing {expected}"); }
        state.count.set(5);
        let next = host.render(&state, &events, &scope);
        assert!(text(&next, "10"));
        assert!(text(&next, "child Root 5"));
        input(&next, "Root").emit(1, "Edited root".into()).unwrap();
        input(&next, "Nested").emit(1, "Edited nested".into()).unwrap();
        input(named(&next, "custom-name"), "Nested").emit(1, "Custom nested".into()).unwrap();
        let (tx, rx) = std::sync::mpsc::channel();
        worker.sender().dispatch(move |state, _| tx.send((state.name.get(), state.counter.name.get())).unwrap()).unwrap();
        assert_eq!(rx.recv_timeout(std::time::Duration::from_secs(2)).unwrap(), ("Edited root".into(), "Custom nested".into()));
    }
}
"#;

/// Identical templates must work with generated and manually authored State.
#[test]
fn direct_values_composables_props_models_and_shadowing_work_in_both_modes() {
    let child = inrux_compiler::compile(
        r#"<view>
<Text text={format!("child {} {}", title, count)} />
</view>
<rust>
use inrux::prelude::*;
pub struct Props { pub title: String, pub count: u32 }
</rust>
"#,
    )
    .unwrap();
    let name_input = inrux_compiler::compile(
        r#"<view>
<Input value={value} @input={on_update_value.clone()} />
</view>
<rust>
use inrux::prelude::*;
pub struct Props { pub value: String, pub on_update_value: Emit<String> }
</rust>
"#,
    )
    .unwrap();
    let automatic = r#"
let count: Signal<u32> = Signal::new(2);
let source = count.clone();
let doubled: Computed<u32> = Computed::new(move || source.get() * 2);
let name: Signal<String> = Signal::new("Root".into());
let enabled: Signal<bool> = Signal::new(false);
let rows: Vec<u32> = vec![2, 3, 0];
let counter: Counter = use_counter();
"#;
    let manual = r#"
#[derive(Clone)]
pub struct State { count: Signal<u32>, doubled: Computed<u32>, name: Signal<String>, enabled: Signal<bool>, rows: Vec<u32>, counter: Counter }
pub fn setup() -> State {
    let count = Signal::new(2);
    let source = count.clone();
    State { count, doubled: Computed::new(move || source.get() * 2), name: Signal::new("Root".into()), enabled: Signal::new(false), rows: vec![2, 3, 0], counter: use_counter() }
}
"#;
    for (tag, initialization) in [("rust", automatic), ("rust explicit", manual)] {
        let source = format!(
            "<view>\n{VIEW}\n</view>\n<{tag}>\n{SUPPORT}\nmod child {{ {child} }}\nmod name_input {{ {name_input} }}\n{initialization}\n{CHECKS}\n</rust>\n"
        );
        let output = crate::typecheck::check_source(&source, true);
        assert!(
            output.status.success(),
            "{tag}: {}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
}
