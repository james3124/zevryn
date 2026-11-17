# Game detail + search screens — Zevryn launcher

Added an offline game-detail screen and a local search screen. No Rust files,
`MainActivity`, build files, CI, or another agent's files were touched. The
parent session owns wiring these into the launcher's tabs/navigation.

## Kotlin (`package com.zevryn.app.screens`)

- **`GameDetailFragment.kt`** — detail screen. Reads `id` / `title` / `kind`
  from its arguments bundle with safe defaults (`newInstance(...)` factory is
  provided). Renders a colored placeholder cover tinted by kind, title,
  description, a **Play Now** button (demo Toast reporting the Rust-modeled
  start-type code: 1403 PC emu / 1407 Steam / 1406 cloud), and a **More Details**
  expandable text toggle. Long-pressing the cover (or the detail page) opens a
  `MaterialAlertDialog` menu reproducing the reference actions: **PC Game
  Settings**, **Check Compatibility**, **Add/Remove Library** (toggles local
  state with a Toast), and **Uninstall** (with a confirmation dialog).
  Account / friends / cloud features are omitted. No native/JNI calls, so it is
  crash-safe even if `libzevryn.so` failed to load.
- **`SearchFragment.kt`** — search screen. An `EditText` filters a small
  hardcoded catalogue as you type; a filter `Spinner` (reusing the existing
  `@array/filter_options`: All / PC / Mobile) filters by platform. Results are a
  focusable `RecyclerView` of kind-tinted rows; tapping shows a demo Toast.
  Input, filtering, and color parsing are all guarded.

Both use only existing dependencies (appcompat, material, recyclerview,
fragment-ktx). Focusable views carry D-pad `nextFocus*` attributes; the search
list uses `com.zevryn.app.FocusHelper` with a single column.

## Layouts (`res/layout/`)

- `fragment_game_detail.xml` — scrollable detail page: `detailBannerContainer` /
  `detailBanner`, `detailKind`, `detailTitle`, `detailDescription`,
  `moreDetailsText`, `moreDetailsToggle`, `playNowButton`, `detailRoot`.
- `fragment_search.xml` — `searchInput`, `searchFilterSpinner`, `searchResults`.
- `item_search_result.xml` — `resultBadge`, `resultTitle`, `resultKind`.

## Strings (`res/values/strings.xml`, appended only)

Added `zd_detail_*` and `zd_search_*` strings (the `zd_` prefix avoids
collisions with the other screens being added in parallel). Values mirror the
GameHub Lite reference strings: "Play Now", "More Details", "PC Game Settings",
"Check Compatibility", "Add to Game Library" / "Remove Library", "Uninstall".
No existing string, color, drawable, or style was modified.

## R-reference verification

All referenced resources resolve against existing/added values:
- layouts: the three added above;
- ids: all declared via `@+id` in the added layouts;
- strings: `zd_*` (added), `unknown_kind`, `game_click_toast` (existing);
- arrays: `@array/filter_options` (existing);
- colors: `windowBackground`, `surface`, `textPrimary`, `textSecondary` (existing);
- drawables: `banner_background`, `kind_chip`, `card_background`,
  `focus_border`, `badge_background`, `spinner_background`, `ic_search`
  (existing).

## Suggested wiring (parent session)

Import `com.zevryn.app.screens.GameDetailFragment` /
`com.zevryn.app.screens.SearchFragment`. Show detail with
`GameDetailFragment.newInstance(id, title, kind)` (e.g. from a library/card
click); show search as a tab or via a search action.
