# M1.4 Config Types and Merge Semantics

## Status

Implemented.

## Goal

This topic teaches how Lune turns several human-authored TOML files into one
effective runtime configuration. It unlocks predictable showcase startup:
built-in defaults establish a complete config, root `Lune.toml` applies
project-wide preferences, and showcase `Showcase.toml` overrides only the
fields that are specific to that showcase.

## Concept

Config has two shapes:

- a complete effective config used by engine code
- partial config layers loaded from files

The effective config has no missing values. If the renderer asks for the clear
color, it should receive `[f32; 4]`, not `Option<[f32; 4]>`. Missing values are
resolved before runtime systems see the config.

Config layers are partial because a file should only mention what it wants to
change. A root `Lune.toml` might set the window title and diagnostics level. A
showcase `Showcase.toml` might set the scene asset path and window size. The
merge turns those partial layers into a complete value.

The accepted order is:

1. built-in defaults
2. root `Lune.toml`
3. showcase `Showcase.toml`

Later layers win over earlier layers. This is simple enough to reason about
while still supporting project-wide defaults.

## Merge Rules

Lune uses these rules for M1:

- Scalars replace earlier values.
- Nested tables merge field-by-field.
- Arrays replace earlier arrays; they do not append.
- Missing fields leave the current value unchanged.
- Unknown fields are errors in strict mode.

Example:

```toml
# Lune.toml
[window]
title = "Lune Dev"
width = 1600

[assets]
search_paths = ["assets", "assets/shared"]
```

```toml
# showcases/smoke/Showcase.toml
[window]
height = 900

[assets]
search_paths = ["showcases/smoke/assets"]
```

The effective config keeps `window.title = "Lune Dev"` and
`window.width = 1600`, overrides `window.height = 900`, and replaces the asset
search path list with only `["showcases/smoke/assets"]`.

## Why Arrays Replace

Appending arrays sounds convenient, but it hides ordering and duplication
problems. Asset search paths, plugin lists, and forced render capability lists
often have order-dependent behavior. If a showcase wants a different list, it
should state the full list.

Replacement is easier to test and easier to explain:

- defaults say what happens with no files
- root says the project-wide list
- showcase says the exact showcase list

If Lune later needs append semantics, that should be an explicit operation
such as `extra_search_paths`, not implicit array merging.

## Unknown Fields

Human config should fail loudly when a field is misspelled. This matters
because TOML files are edited by hand and a typo silently ignored can waste a
lot of debugging time.

For M1, config parsing is strict. Unknown root tables and unknown nested fields
are errors. This means:

```toml
[window]
widht = 1920
```

must fail instead of silently leaving `width` at the default value.

## Alternatives

| Alternative | Pros | Cons | When It Fits |
| --- | --- | --- | --- |
| One required config file | Simple loader | Every showcase repeats boilerplate | Small apps |
| Defaults plus one override file | Easy to understand | No project-wide preferences | Single-demo projects |
| Defaults + root + showcase | Clear hierarchy; little boilerplate | Requires merge rules | Lune M1 |
| Deep merge arrays by append | Convenient for additive lists | Duplicates and order bugs are easy | Rare explicit extension points |
| Use raw `toml::Value` everywhere | Flexible | Runtime code loses typed contracts | Tooling prototypes |
| Strong structs plus partial layers | Typed runtime config; clear merge behavior | More structs to maintain | Engine/runtime config |

## Industry Examples

Game engines usually have multiple configuration sources. Unreal has project
settings, platform settings, command line overrides, and `.ini` layering.
Unity has project settings and per-scene or per-asset settings. Exact merge
rules differ, but the common pattern is a complete runtime view built from
several authored sources.

Rust services often use the same model with defaults, file config, environment
variables, and command line flags. Lune intentionally stops before environment
and command line layers in M1 because the architecture decision says those are
future layers.

## Lune Architecture Impact

`lune_config` is a base crate. It should not depend on renderer, platform,
asset, ECS, or showcase crates. Higher-level crates receive an effective config
or their own extracted subsystem config.

M1 only models generic engine startup fields:

- app name
- diagnostics config from `lune_diagnostics`
- window title, size, and vsync
- renderer backend and clear color
- asset search paths

Later milestones can add subsystem-specific config once those subsystems
exist. This keeps M1 focused on parse and merge semantics, not on predicting
every future renderer or asset option.

## Decision

Add `lune_config` with two public shapes:

- `LuneConfig`: complete effective config
- `ConfigLayer`: partial TOML-deserializable layer

`lune_config` owns `DiagnosticsConfig` and the diagnostics enum types because
configuration belongs in this crate. `lune_diagnostics` consumes that config
when it initializes tracing. Diagnostics enum values deserialize from
lower-case TOML strings such as `"debug"`, `"compact"`, and `"enabled"`.

Implement:

- `LuneConfig::default()`
- `parse_config_layer(source, toml_text)`
- `merge_config_layers(layers)`
- `merge_config_layers_onto(config, layers)`

`merge_config_layers` starts from built-in defaults and applies the supplied
layers. `merge_config_layers_onto` applies layers onto an existing effective
config, which keeps custom-base tests and future in-memory config editing
explicit. The lower-level `apply_config_layers` helper stays private so the
public API names describe the merge contract clearly.

## Failing Tests

The tests live in `crates/lune_config/tests/config_layers.rs`.

They cover:

- built-in defaults
- TOML deserialization for every current nested table
- diagnostics enums deserializing from TOML strings
- unknown root fields
- unknown nested fields
- parse errors preserving source names
- defaults + root + showcase precedence
- nested table field-by-field merging
- array replacement
- empty layer behavior

## Implementation Handles

- Key types: `LuneConfig`, `ConfigLayer`, nested effective config structs,
  nested partial layer structs, `ConfigError`.
- Key invariant: runtime code should consume complete config, not partial
  config layers full of `Option`.
- Merge order: apply layers from left to right; later `Some` values replace
  current effective values.
- Error cases: TOML syntax errors and unknown fields.
- Edge cases: empty layers, nested partial tables, array replacement, source
  names in parse errors.
- Useful assertions: arrays never append; missing nested fields preserve the
  previous effective value.

## Verification

Run the full local verification command:

```bash
just check
```

`just check` should pass.

## Follow-Up Topics

- Config file discovery from workspace/showcase paths
- CLI and runtime config layers
- Renderer capability forcing config
- Asset search path and scene selection config
