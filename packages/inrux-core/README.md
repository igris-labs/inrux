# inrux-core

Backend-independent application state, typed events, dependency injection, reactive values, instance identity, and bounded background work. This crate uses the Rust standard library and has no renderer dependency. Applications usually access it through `inrux`.

## Reactive values

```rust
use inrux_core::reactive::{Computed, Signal};

/// A computed getter evaluates lazily and tracks only the sources it reads.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let count = Signal::new(2_u32);
    let source = count.clone();
    let doubled = Computed::new(move || source.get() * 2);
    assert_eq!(doubled.get()?, 4);
    count.update(|value| *value += 1);
    assert_eq!(doubled.get()?, 6);
    Ok(())
}
```

Cloning a Signal shares its value; calling Signal::new creates independent state. Computed getters must be pure. Conditional and nested reads are tracked; cycles and concurrent evaluation return ReactiveError. Signal writes notify subscriptions after releasing the value lock.

## Serialized application work

```rust
use inrux_core::Worker;
use std::{sync::mpsc, time::Duration};

/// Commands mutate state on the coordinator and report bounded-queue overload.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let worker = Worker::spawn(0_u32, 64, || {})?;
    let (tx, rx) = mpsc::channel();
    worker.sender().dispatch(move |state, _context| {
        *state += 1;
        let _ = tx.send(*state);
    })?;
    assert_eq!(rx.recv_timeout(Duration::from_secs(2))?, 1);
    Ok(())
}
```

A command receives `&mut State` and `&Context<State>`. Context is confined to the coordinator; background jobs receive owned inputs and Cancellation. `spawn_task(work, apply)` uses two workers and 16 pending slots. Results use a separate bounded queue and return to the coordinator. Task errors and panics become typed outcomes; a coordinator panic stops the coordinator and requests task cancellation.

## Ownership and scopes

`Instances<T>::mount` returns a typed generation-bearing handle. `with`/`update` reject stale handles; `unmount` invalidates old events and owned work before slot reuse. `Sender::for_instance` routes child commands through the root queue. `spawn_owned_task` explicitly binds work to an instance.

`Scope::provide(value)` creates a child scope and `inject::<T>()` returns the nearest `Option<Arc<T>>`. Provide a Signal or reactive service when changes should propagate. Plain values remain immutable dependencies.

## Validate and extend

```sh
cargo test -p inrux-core --locked
cargo doc -p inrux-core --no-deps
```

Tests cover ordering, saturation, cancellation, panic behavior, retained state, subscriptions and computed dependencies. Snapshot clones containing Signals/Instances share those handles; native rendering instead consumes owned Node descriptions.

Render refs will reuse generational identity and cancellation while keeping native widget access on the UI thread. They are [not implemented yet](../../docs/STATUS.md); they are distinct from a Rust borrow or InstanceHandle.
