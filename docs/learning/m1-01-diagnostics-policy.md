# M1.1 Diagnostics Policy

## Status

Draft tests written. User implementation pending.

## Goal

This topic teaches the split between typed library errors and structured engine
telemetry. It unlocks a shared diagnostics crate that every subsystem can use
without each crate inventing its own logging and error conventions.

## Concept

Diagnostics has two separate jobs:

- Errors report a specific failed operation to the caller.
- Logs and spans report runtime behavior to a human or tool.

Library crates should return typed errors. A typed error is an enum or struct
that preserves meaning in the type system. Callers can match on it, attach
context, retry selected cases, or convert it into their own error type. Lune
uses `thiserror` for these library errors because it derives standard `Error`
and `Display` implementations without hiding the variant shape.

Application and tool entry points may use `anyhow` when the caller is a human
at the edge of the program. A showcase binary can use `anyhow::Result<()>`
because it usually only needs to print a chain of causes and exit. Library
crates should not expose `anyhow::Error` in public APIs because that erases
the domain contract.

Tracing is different from errors. A function can succeed and still need to
emit useful facts: which renderer backend was selected, which config file won
a merge, which asset path failed to reload, or how long a frame stage took.
Lune uses `tracing` so events can carry fields and spans instead of plain text.
Spans describe nested work, such as `frame`, `extract`, `upload`, and `draw`.
Events describe facts inside those spans.

The older `log` facade is still common in Rust dependencies. Lune engine code
should emit through `tracing`, but the diagnostics setup may install a
`tracing-log` bridge so dependency `log` records are not lost.

### What Tracing Is For

Use tracing when you need to answer questions about what the engine did while
it was running:

- Which config files were loaded?
- Which renderer backend and GPU were selected?
- Which frame stage is slow?
- Which asset path failed to reload?
- Which ECS schedule or render pass emitted an error?

Plain text logging can answer some of this, but it usually loses structure.
Tracing keeps names, fields, and nesting. That matters in an engine because a
single frame can touch many systems. The useful question is often not "did a
line print?" but "inside which frame, pass, entity, asset, or subsystem did
this event happen?"

The core tracing vocabulary:

- Event: one fact that happened at a point in time.
- Span: a named region of work that can contain events and child spans.
- Field: structured data attached to an event or span.
- Subscriber: the process-wide collector that receives spans and events.
- Layer: part of a subscriber stack, such as formatted terminal output.
- Filter: runtime rule deciding which spans and events are enabled.

Libraries emit spans and events. Binaries initialize the subscriber.

That boundary is important. `lune_render`, `lune_asset`, and `lune_core` should
not decide how output is formatted or where it goes. They only emit diagnostics:

```rust
use tracing::{debug, info, info_span, warn};

fn load_scene(path: &str) {
    let span = info_span!("load_scene", path);
    let _entered = span.enter();

    info!("reading scene metadata");
    debug!("resolving mesh handles");
    warn!(missing_material = "DefaultMaterial", "using fallback material");
}
```

If no subscriber is installed, these calls are cheap and mostly inert. Once a
showcase installs diagnostics, the same library code starts producing useful
output without changing library APIs.

### Initialization Boundary

Tracing setup is global process state. Initialize it once, as early as possible
in a binary entry point:

```rust
use lune_config::{DiagnosticsConfig, DiagnosticsLevel};
use lune_diagnostics::init_diagnostics;

fn main() -> anyhow::Result<()> {
    init_diagnostics(
        DiagnosticsConfig::developer().with_level(DiagnosticsLevel::Info),
    )?;

    tracing::info!(target: "lune_showcase", "starting renderer smoke test");

    Ok(())
}
```

Do not initialize diagnostics from a library crate. Library initialization
would make dependency order surprising and would prevent the final application
from choosing its own format, verbosity, or telemetry backend.

For Lune, initialization belongs in:

- showcase binaries
- future tools such as `lune_cooker`
- integration test harnesses that need diagnostic output

It does not belong in:

- `lune_render`
- `lune_asset`
- `lune_core`
- constructors for engine subsystems
- individual tests that run in parallel unless the test owns global setup

### What `init_diagnostics` Should Do

M1.3 should turn `lune_config::DiagnosticsConfig` into a tracing subscriber:

1. Convert `DiagnosticsLevel` into a root verbosity directive.
   `Trace` maps to `trace`, `Info` to `info`, `Warn` to `warn`, and `Error` to
   `error`.
2. Build a `tracing_subscriber::EnvFilter` from that directive.
3. Build a formatted subscriber using `tracing_subscriber::fmt`.
4. Apply `LogFormat::Compact` or `LogFormat::Pretty` to the formatter.
5. If `LogBridge::Enabled`, install `tracing_log::LogTracer` so dependency
   `log` records are forwarded into tracing.
6. Install the subscriber with a fallible global init API.
7. Return `DiagnosticsError::AlreadyInitialized` if global setup already
   happened.
8. Return `DiagnosticsError::LogBridgeInstall` if the `log` bridge cannot be
   installed.

This first API intentionally uses a typed `DiagnosticsLevel` instead of an
arbitrary filter string. That keeps config simple while the engine is young.
Later, Lune can add crate-specific directives such as `lune_render=trace,warn`
when there is a real use case for subsystem-specific filtering.

### When To Emit Events And Spans

Use an event for a single meaningful fact:

```rust
tracing::info!(backend = "vulkan", "renderer backend selected");
tracing::warn!(path = "assets/scene.gltf", "asset reload failed");
tracing::error!(error = %err, "failed to create swapchain");
```

Use a span when later events need context:

```rust
let frame = tracing::info_span!("frame", frame_index);
let _frame = frame.enter();

let extract = tracing::debug_span!("extract_render_data");
let _extract = extract.enter();
```

Prefer fields over formatting everything into the message. This keeps data
queryable later if Lune writes logs to JSON, a file, or a telemetry UI.

Good:

```rust
tracing::debug!(entity = entity.index(), generation = entity.generation(), "spawned entity");
```

Less useful:

```rust
tracing::debug!("spawned entity {}:{}", entity.index(), entity.generation());
```

### Level Policy

Use levels consistently:

| Level | Meaning in Lune | Example |
| --- | --- | --- |
| `Trace` | Very detailed flow, usually too noisy for normal development | Per-resource renderer decisions |
| `Debug` | Useful while developing a subsystem | Chosen config layer or renderer capability |
| `Info` | Normal startup and major lifecycle facts | Window created, renderer selected |
| `Warn` | Recoverable problem or fallback | Reload failed, keeping old asset |
| `Error` | Operation failed and needs caller/user attention | Could not create device |

The first public enum includes `Trace`, `Debug`, `Info`, `Warn`, and `Error`.
`Info` remains the default because normal local runs should show lifecycle
facts without dumping subsystem internals.

## Alternatives

| Alternative | Pros | Cons | When It Fits |
| --- | --- | --- | --- |
| String errors | Very simple to write | Callers cannot match cases; context is easy to lose | Throwaway binaries |
| `anyhow` everywhere | Fast iteration; good context chains | Public library APIs lose typed failure contracts | App entry points, CLIs, tests |
| Custom `Error` impls by hand | Full control | Repetitive and easy to format inconsistently | Rare errors with unusual display needs |
| `thiserror` typed enums | Clear public contract; little boilerplate | Requires designing variants deliberately | Lune library crates |
| `log` only | Simple facade; many crates support it | No spans; structured fields are limited | Small applications or dependency compatibility |
| `tracing` | Spans, structured fields, async-aware ecosystem | Setup is more complex than `println!` or `log` | Engine runtime diagnostics |

## Industry Examples

Rust engine and infrastructure crates commonly keep library errors typed and
leave erased error handling to binary edges. This is public Rust ecosystem
practice rather than a single-engine rule.

Bevy uses structured diagnostics and tracing-compatible instrumentation in
engine code. Exact internal policies vary by version, but the broad pattern is
publicly visible: engine systems produce structured diagnostic data, while app
startup decides how to present it.

Large engines outside Rust use the same separation in different forms. Unreal
and Unity have structured categories, verbosity levels, and runtime filtering
for logs. Their C++ and C# error models differ from Rust, so the direct mapping
to `thiserror` is an inference, but the separation between recoverable domain
failures and runtime telemetry is the same engine concern.

## Lune Architecture Impact

`lune_diagnostics` sits at the bottom layer. It must not depend on renderer,
asset, ECS, or platform crates. Higher-level crates depend on it for common
diagnostic setup and shared error conventions.

The first public API is deliberately small:

- `lune_config::DiagnosticsConfig` describes level, format, and `log` bridge
  policy.
- `lune_config::DiagnosticsLevel` keeps root verbosity typed as trace, info,
  warn, or error.
- `init_diagnostics` installs the process-wide subscriber.
- `DiagnosticsError` reports duplicate initialization and bridge setup failures
  as typed errors. If Lune later accepts arbitrary filter directives, invalid
  directive syntax should also be reported as a typed diagnostics error.

Future config work uses the same policy: config parsing and merge failures are
typed library errors, while showcase binaries may convert them to `anyhow`
when printing final startup failure context.

## Decision

Lune libraries use `thiserror` for public typed errors. Showcase binaries and
tools may use `anyhow` at the program edge. Engine code emits spans and events
through `tracing`. The diagnostics setup may bridge dependency `log` records
with `tracing-log`.

The default local developer diagnostics config should be human-readable:

- level: `DiagnosticsLevel::Info`
- format: compact
- `log` bridge: enabled

## Failing Tests

The initial tests live in
`crates/lune_diagnostics/tests/diagnostics_api.rs`.

They define this behavior:

- developer defaults are stable and human-readable
- config builder methods override independent fields
- diagnostics level defaults to `Info`
- first valid initialization installs the process-global subscriber
- second initialization returns `DiagnosticsError::AlreadyInitialized`

The tests are expected to fail at `todo!()` boundaries until the user
implements tracing setup.

## Implementation Handles

- Key types: `lune_config::DiagnosticsConfig`, `DiagnosticsLevel`,
  `LogFormat`, `LogBridge`, and `DiagnosticsError`.
- Key invariant: diagnostics subscriber setup is process-global and should be
  installed through one controlled API.
- Error cases: duplicate global subscriber setup and log bridge installation
  failure.
- Edge cases: tests run in parallel, dependencies may emit `log` records
  before Lune initializes, repeated initialization must not panic.
- Useful assertions: a valid first setup should return `Ok(())`; a second
  setup should report `DiagnosticsError::AlreadyInitialized`.

## Verification

Before implementation, this command should compile and then fail in nextest at
the intentional `todo!()` tests:

```bash
just check
```

After implementation, `just check` should pass.

## Follow-Up Topics

- M1.3 tracing subscriber setup
- M1.4 config parsing and merge semantics
