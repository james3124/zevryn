# Area report — input mapping / `.icp` profiles (Rust port)

Reference (read-only): `sources/com/xj/mapping/` (538 files) and
`apktool/assets/inputcontrols/profiles/*.icp` of GameHub Lite v5.1.8, plus
`sources/com/xj/common/data/list/TopPlatFormType.java` for the 8-value platform
enum. All modules are dependency-free (`std` only) with `#[cfg(test)]` tests.

## Files created

| File | Models | Contents |
|---|---|---|
| `mod.rs` | package root + `.icp` model | private JSON reader; `ElementType`, `Shape`, `Element`, `Profile`, `ProfileError`; `Profile::parse`/`validate`/`element_count`; submodule declaration + re-export-free root; tests |
| `keymap.rs` | `TopPlatFormType` + binding vocabulary | `TopPlatform` (8 values, `ALL`, `from_str`/`as_str`); `BindingKind`; `classify_binding`; `ResolvedBinding`; `resolve`; `resolve_profile`; tests |

Wiring (`app/src/lib.rs`): declare the package root with an explicit path, e.g.
`#[path = "../sources/com/xj/mapping/mod.rs"] pub mod mapping;`. `keymap.rs` is
resolved automatically as a sibling of `mod.rs`, and it uses
`super::{Element, ElementType, Shape, Profile}` so both modules share the exact
same types. No other crate-level module is required — the JSON reader is
vendored privately inside `mod.rs`.

## Fidelity to the existing core crate

Names and behavior mirror `/home/Project/Zevryn/src/lib.rs` and
`/home/Project/Zevryn/src/keymap.rs` exactly: same variants (`DPad`, `Button`,
`Stick`, `RangeButton`, `Other`; `Circle`, `Square`, `Other`; the 8
`TopPlatform` values), same binding arity rule (4 for D-Pad/Button/Stick, 6 for
RangeButton), same `classify_binding` token rules and same `ResolvedBinding`
shape. Additions are purely additive helpers (`element_count`, `resolve_profile`,
`TopPlatform::ALL`/`as_str`, a `rejects_wrong_binding_arity` test).

## Observations from the real assets

- 4 bundled profiles: `controls-1.icp`, `controls-2.icp`, `controls-100.icp`,
  `controls-200.icp`. All parse and validate under the modeled rules.
- Element `type` tokens observed: `BUTTON` (66), `STICK` (5), `D_PAD` (3),
  `RANGE_BUTTON` (2).
- `shape` tokens observed: `CIRCLE`, `SQUARE`, `RECT`, `ROUND_RECT`. The model
  keeps the original `Shape { Circle, Square, Other }`; `RECT`/`ROUND_RECT`
  deliberately fall into `Other` (shape is not used by `validate`, so this is
  lossless for the portable logic and keeps type names identical to the core
  crate).
- `range` token observed once: `FROM_0_TO_9` on a `RANGE_BUTTON`.
- Binding tokens are `GAMEPAD_*`, `KEY_*` and `MOUSE_*` plus `NONE`. Under the
  reference `classify_binding`, `KEY_*` tokens become `Unknown` and
  `MOUSE_*_BUTTON` becomes `Button` — the port preserves that behavior rather
  than inventing new `BindingKind` variants.

## Intentionally not implemented

- Android/JNI/UI: touch dispatch (`interaction/**`), inject/socket/touch utils,
  view binding, adapters, permission flows, AI/huiwan integration, database,
  download/update logic. None of these are portable.
- Key-code constant tables: see `../xiaoji/sdk/REPORT.md` — `GcmKeyCode` /
  `HidKeyCode` constants were stripped by JADX.

## Verification

Compiled and tested standalone with `rustc --edition 2021 --test` using a
temporary harness that declares both `mod.rs` files; **16 tests pass, 0
warnings**. Real-asset tests read the absolute reference path and return early
if it is absent, so the suite also passes in a checkout without the reference
tree.
