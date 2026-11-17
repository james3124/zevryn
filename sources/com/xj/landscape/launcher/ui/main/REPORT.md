# Report — Rust reimplementation of the launcher dashboard area

Write area: `app/sources/com/xj/landscape/launcher/ui/main/`
Reference (read-only): `reference/gamehub-lite-v518-decompiled/gamehub-lite-v518-decompiled/sources/com/xj/landscape/launcher/ui/main/`
Baseline reused: `src/dashboard.rs` (same types/approach, adapted into the mirrored area).

## Files written

| File | Purpose | Mirrors |
|---|---|---|
| `mod.rs` | Area module root; declares and re-exports all submodules. | (wiring) |
| `me/mod.rs` | `CardLineType` (1–5 + `Unknown`), `GameCard`, `CardLine`, `Dashboard` + JSON `parse`, `TopPlatform`/`top_platform_from_str`, `entry_id` constants, `GameCard` id/URL predicates. | `me/MyGamesData.java`, `me/MyGamesItemData.java`, `com/xj/common/data/list/CardLineData.java`, `com/xj/common/data/list/CardLineType.java` |
| `me/json.rs` | Local, dependency-free JSON reader (copy of the core crate's `src/json.rs`) kept inside the area so it is self-contained regardless of `#[path]` wiring. | (helper) |
| `focus.rs` | `Direction`, `FocusGraph` (generic node/edge state machine from `src/dashboard.rs`), `FocusNode`, `launcher_focus_graph()` with the exact D-pad edges of `d(event, lastFocusedView)` / L1–R1 handling in `b(event)`. | `LandscapeLauncherMainActivity.java` (focus methods), `LlauncherActivityNewLauncherMainBinding` |
| `tabs.rs` | The two tabs (Dashboard → `MyFragment`, Instant Play → `CloudPlayFragment`), `TabId`, `FragmentKind`, `Tab`, `build_tabs(...)`, `locale_gate_allows_instant_play()`. | `LandscapeLauncherMainActivity.java` (`initView`, tab list), `TabItemData.java`, `com/xj/base/language/GHLocaleManager.java` (`m()` = locale language `"zh"`) |
| `MainAdapterKt.rs` | `ViewHolderKind` + `view_holder_for()`, `fixed_image_view_type_key()`, `should_show_line()` (`j()`), `is_same_vertical_type()` from `setModels`. | `MainAdapterKt.java`, `me/MyContainerView.java` |

All modules use only `std`; no Cargo dependencies were added. `#[cfg(test)]` unit tests are included in every module (22 tests total, all passing, zero warnings in an isolated verification build).

## Key behaviors modeled
- `CardLineType.fromCode` / `code` for codes 1–5 and `Unknown(-1)` fallback (retains offending code).
- Dashboard parsing of `card_line_data` (bare array or wrapped object), with the decompiled DTO field names.
- Top-platform classification from `topPlatformType` strings and the `CardLineData.Companion` entry ids — including the quirk that PC_LINK/PS_LINK collapse to `Other`.
- D-pad focus graph over `mainTabView`, `flOpenNav`, `llRightTopStatus`, `cards`, with unhandled directions returning no move (Java `null`).
- Two-tab construction with the `GHLocaleManager.m()` locale gate: Instant Play is appended unconditionally when `AppConfig.l()` is set, otherwise only when the locale language is `zh`.
- Adapter dispatch: card-type → holder mapping, fixed-image view-type key (`class_<size>_<ratio>`, defaults 3 / 0.56), empty-fixed-image row filtering, and vertical-row merge predicate.

## Not implemented (out of scope / non-portable)
- **Android views / UI runtime**: `Activity`, `Fragment`, `FocusableRecyclerView`, `LauncherTabLayout`, `MyContainerView`, all `viewholders/*`, `ShapeView`/`ShapeTextView`, DataBinding, resources and layouts. Only their portable decision logic is modeled.
- **RecyclerView / adapter machinery**: `MultiTypeAdapter`, `GridLayoutManager`, `DiffUtil` (`MyGamesDiffCallback`), item decorations, `notifyItemChanged`, smooth scrolling, child-list views (`GameChildListView`, `TopPlatformChildListView`). The `setModels` row-merging rule is captured as a pure predicate only.
- **Networking / coroutines (MyVM)**: `getMyGames`, `fetchGameDetail`, `fetchGameStartupParams`, `syncDownloadInfo4MyGames`, repository calls, Kotlin flows/channels and lifecycle observers. DTO parsing is modeled; transport is not.
- **Game launching / platform SDK**: `GameAppLauncherHelper`, `LaunchTypeIconKt`, operation popups, download state (`CommonDownloadTaskState`), `GameLibraryTable`/`GameDetailEntity` internals.
- **Locale/OS integration**: real `Context`, `LanguageUtils`, `AppPreferences`, `AppConfig` values — represented by plain boolean/string inputs instead.
- **Cloud / Steam subsystems**, animations, image loading and all vendor SDK code.

## Wiring note for the orchestrator
`app/src/lib.rs` is intentionally untouched. Add e.g.
```rust
#[path = "sources/com/xj/landscape/launcher/ui/main/mod.rs"]
pub mod launcher_main;
```
to compile this area. The `me/mod.rs` module is also standalone (it embeds its own JSON reader via `#[path]`), so it can be wired directly as `.../main/me/mod.rs` if preferred.
