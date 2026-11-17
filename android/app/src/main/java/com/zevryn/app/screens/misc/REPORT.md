# Misc screens — Zevryn Android launcher

Three offline-safe screens added under `screens/misc/`. No Rust sources,
`MainActivity`, `build.gradle`, CI or other agents' files were touched. The
parent session owns wiring these into the `ViewPager2` tabs.

Grounding (in-scope references only): MobileGameV4Activity (mobile games tab),
pcvirtualbtn DisplayActivity / ControlsProfileListActivity (on-screen virtual
buttons), PreviewImageActivity / ProductDocActivity / menu (About).

## Kotlin (package `com.zevryn.app.screens.misc`)

- **`MobileGameFragment.kt`** — "Mobile Games" tab. A `RecyclerView` of five
  installed games backed by static local demo data; tapping a row shows a
  `Selected <game> (demo)` Toast. Single-column D-pad focus via
  `com.zevryn.app.FocusHelper`. No package-manager queries or native calls.
- **`VirtualButtonsFragment.kt`** — "Virtual Buttons" tab. A focusable on-screen
  control overlay: a D-pad (up/down/left/right + OK) and A/B/X/Y circle
  buttons, each a `focusable` `TextView` whose background is an oval
  `GradientDrawable` built in code (no new drawable files). A `SwitchCompat`
  toggles overlay visibility; pressing any button shows a `<label> pressed
  (demo)` Toast. Explicit next-focus neighbours wire the D-pad cross, bridge
  it to the A/B/X/Y cluster, and form a 2x2 focus grid there.
- **`AboutFragment.kt`** — "About" tab. Displays `Zevryn 0.1.0`,
  `Rust core offline-capable`, a static module-count string, and a short note
  that Zevryn is a GameHub Lite reimplementation. Static resource text only.

## Layouts (`res/layout/`, all `misc_` prefixed)

- `misc_mobile_game.xml` — header, hint, `miscGameRecycler`.
- `misc_item_game.xml` — game badge, name, metadata row.
- `misc_virtual_buttons.xml` — header, hint, `miscOverlaySwitch` and the
  `miscOverlayRoot` surface containing the D-pad and A/B/X/Y clusters.
- `misc_about.xml` — scrollable stack of version/core/modules/note text.

## Strings (`res/values/strings.xml`, appended only)

Added tab labels (`misc_tab_*`) plus `misc_mobile_*`, `misc_game_*`,
`misc_virtual_*`, `misc_dpad_*`, `misc_btn_*`, `misc_overlay_*` and
`misc_about_*`. No existing string was modified; no new colors or drawables
were required. All widgets use existing AppCompat/Material/RecyclerView
classes already on the classpath.

## Verification

All `R.string.*`, `R.layout.*`, `R.id.*`, `R.color.*` and `R.drawable.*`
references used by these screens were checked against the existing resources
before writing; only missing `misc_*` strings were appended.
