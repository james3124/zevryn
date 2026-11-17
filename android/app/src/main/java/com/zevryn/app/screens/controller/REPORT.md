# Controller screens — Zevryn Android launcher

Added two missing, offline-safe controller screens plus their layouts and
strings. Work was limited to this `controller/` directory, `res/layout/ctrl_*`,
and appended `ctrl_*` strings in `res/values/strings.xml`. No Rust,
`MainActivity`, `build.gradle`, or CI files were touched.

The Rust core has gamepad/button-mapping models but exposes no JNI calls for
them, so both screens are **display/demo only** (toasts, local hardcoded data)
and cannot crash when the native library is unavailable.

## Kotlin (package `com.zevryn.app.screens.controller`)

- **`GamepadFragment.kt`** — inflates `ctrl_gamepad.xml`. Shows a focusable
  controller schematic (D-pad Up/Down/Left/Right and face buttons A/B/X/Y, each
  with explicit `nextFocus*` neighbours), a `Calibrate` button that shows a
  toast, and a `RecyclerView` of mapped buttons from local demo data.
- **`ButtonMappingFragment.kt`** — inflates `ctrl_button_mapping.xml`. A
  `RecyclerView` of actions (Move, Jump, Attack, Sprint, Interact, Pause) with
  a bound-key label; tapping a row shows `Press a key...` (no real capture).
  Also hosts the shared top-level `CtrlBinding` model and `CtrlBindingAdapter`
  (used by both lists) so the two screens stay small and consistent.

Both lists use `com.zevryn.app.FocusHelper` with a single column so D-pad focus
walks up/down the rows.

## Layouts (`res/layout/`, all `ctrl_` prefixed)

- `ctrl_gamepad.xml` — header/hint, focusable schematic with D-pad and face
  buttons, `ctrlCalibrateButton`, a "Mapped buttons" label, and
  `ctrlGamepadBindings`.
- `ctrl_button_mapping.xml` — header/hint and `ctrlBindingRecycler`.
- `ctrl_item_binding.xml` — one action-name + bound-key row
  (`ctrlActionName` / `ctrlBoundKey`), shared by both fragments.

All widgets use existing AppCompat/RecyclerView classes from current
dependencies, and every `@color` / `@drawable` reference already exists
(`card_background`, `focus_border`, `windowBackground`, `textPrimary`,
`textSecondary`).

## Strings (`res/values/strings.xml`, appended only)

Added `ctrl_*` strings for headers, hints, the schematic button labels,
calibration/toast feedback, action names, and bound-key names. No existing
string was changed and no new colors or drawables were required.

## Wiring note for the parent session

These fragments are not wired into `MainActivity` here (out of scope). Add tabs
as desired, e.g. map positions to `GamepadFragment()` / `ButtonMappingFragment()`
with new tab strings owned by the parent session.
