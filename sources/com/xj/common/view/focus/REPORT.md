# Focus system — Rust reimplementation report

Portable, std-only Rust model of GameHub Lite v5.1.8's custom focus system.
The code lives under `app/sources/com/xj/common/view/focus/` and is fully
self-contained: it does not touch the simpler `FocusGraph` in
`sources/com/xj/landscape/launcher/ui/main/focus.rs`.

## Files

| File | Contents |
| --- | --- |
| `mod.rs` | Module root; declares the submodules and re-exports the public API. |
| `direction.rs` | `FocusDirection { Left, Right, Up, Down }` with `opposite()`, axis helpers, `step()` and `Display`. |
| `event.rs` | `FocusEvent` (direction + `KeyAction`) and `KeyAction` (`ACTION_DOWN/UP/MULTIPLE` codes and labels). |
| `graph.rs` | `NodeId`, `FocusableNode` trait (`id` + `neighbor(direction)`), concrete `FocusNode`, and `FocusGraph` with `add_node`/`link`/`neighbor`. |
| `controller.rs` | `BoundaryPolicy { Clamp, Wrap }`, `Axis`, `FocusController` (explicit edges first, then ordered-axis step with wrap/clamp), `GridFocusController` (row/column movement). |
| `tab.rs` | `AutoScrollMode`, `Visibility`, and `FocusTabModel` (selected/last/default index, clamping, `next_index`, `request_focus`). |
| `state.rs` | `FocusState` with current node id + history, `move_focus(direction)`, `back()`, `set_current()`. |
| `REPORT.md` | This report. |

Every source file carries `#[cfg(test)]` unit tests covering direction/opposite,
graph navigation, wrap/clamp controller behavior (linear and grid), tab focus
and state history. All tests use only `std`.

## Sources studied (read-only)

All under
`reference/gamehub-lite-v518-decompiled/gamehub-lite-v518-decompiled/sources/com/xj/common/view/focus/`:

- `focus/FocusDirection.java` — enum values; only the four navigation
  directions are portable (`L1`, `R1`, `BUTTON_A`, `X` are button events).
- `focus/FocusEvent.java` — direction + action code; the `ACTION_*` codes/labels
  become `KeyAction`.
- `focus/FocusableView.java` — `getFocusableViews()` / neighbor resolution
  (`Companion.d`, `DefaultImpls.b/d/k`) → pure graph neighbor lookup.
- `focus/FocusableRoot.java` — `DefaultImpls.b/d/e`: vertical `index ± 1`
  navigation with `getOrNull` (clamp), first-view default, last-focused view.
- `focus/controller/RecyclerViewFocusController.java` — chooses grid vs
  linear controller; default intercept callbacks return `false`.
- `focus/controller/LinearRecyclerViewFocusController.java` — forward/backward
  scan over focusable positions, handoff at the end (`d()`/`e()`/`f()`) →
  ordered-axis stepping with clamp.
- `focus/controller/GridRecyclerViewFocusController.java` — LEFT/RIGHT within
  a span group (row), UP/DOWN by span group/span index (column), boundary
  intercept → grid row/column resolution with clamp/wrap.
- `focus/view/FocusTabLayout.java` — `setSelectedTab(int)` clamping and
  `-1` sentinel, `y(direction)` (LEFT/RIGHT index ± 1, UP/DOWN unhandled),
  `M(direction)` request-focus → `FocusTabModel`.
- `focus/view/AutoScrollMode.java` — `shouldScrollToPosition` visibility
  policy → `AutoScrollMode::should_scroll`.
- `focus/view/AdapterDelegate.java` — RecyclerView adapter delegation; no
  portable focus logic (out of scope).
- `focus/app/FocusableActivity.java` — implements `FocusableRoot`, holds the
  last focused view and delegates event handling to `FocusableRoot.DefaultImpls`.

## Out of scope

- Android `View` / `ViewGroup`, `HorizontalScrollView`, focus borders and
  `ShapeTextView` / `ShapeView` drawing (`FocusTabLayout.B/G/H/v/w`).
- `android.view.KeyEvent` itself (replaced by `KeyAction`), `KeyEventDispatcher`
  and the `Function1`/`Function2` listener plumbing.
- RecyclerView, `GridLayoutManager` span-size lookup and actual smooth
  scrolling / `smoothScrollToPosition` animations.
- Gamepad button directions (`L1`, `R1`, `BUTTON_A`, `X`) and coroutine/Flow
  notification side effects (`FocusableActivity.s1()` / `s()`).
- Fragment/Activity lifecycle and context/fragment lookup
  (`FocusableView.DefaultImpls.m/g/i`).

## Notes

- `move` is a Rust keyword, so the state transition method is named
  `move_focus`; it is the direct equivalent of Java's `move(direction)`.
- `BoundaryPolicy::Clamp` is the default behavior of the decompiled code
  (boundary returns `null` / intercept hands the event off); `Wrap` is offered
  as the alternate modular policy.
- The launcher's pre-existing `FocusGraph` is left untouched; this module is a
  separate, fuller system. Wiring into `app/src/lib.rs` is handled elsewhere.
