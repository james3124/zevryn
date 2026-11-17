# Area report — app entry / deeplink / navigation (Rust port)

Reference: `sources/com/xj/{app,common/deeplink,common/service}/` of GameHub Lite v5.1.8.
All modules are dependency-free (std only), plain structs/enums, with `#[cfg(test)]`
unit tests. Layout follows `PLAN.md`: Rust `mod.rs` + sibling `.rs` files sit at the
mirrored Java package path, to be wired via `#[path]` in `app/src/lib.rs`.

## Files created

| File | Models | Contents |
|---|---|---|
| `common/deeplink/gh_deep_link_page_type.rs` | `GhDeepLinkPageType` | `DeepLinkPage` enum (7 values), `ALL`, `from_str`/`as_str`, `Display` |
| `common/deeplink/game_hub_deep_link_manager.rs` | portable part of `GameHubDeepLinkManager` | `DeepLink`, `ParseError`, `parse` for `gamesir://gamehub?page_type=...`, percent-decoding |
| `common/deeplink/mod.rs` | package root | submodule declarations + re-exports |
| `common/service/nav_service.rs` | TheRouter `*NavService` interfaces | `NavService` trait, `NavRequest`, `NavOutcome`, `NavError`, `dispatch`, `RecordingNavService` |
| `common/service/mod.rs` | package root | includes the sibling `deeplink` module (single page-type definition), re-exports |
| `app/boot.rs` | `App` -> `SplashActivity` -> main | `AppBootPhase` (4 phases), `BootStep`, `BOOT_STEPS`, `BootSequence` state machine |
| `app/mod.rs` | package root | submodule declaration + re-exports |

The deep-link parser and page enum copy the approach used in the core crate's
`src/deeplink.rs` (same variants and wire strings) so the two stay compatible;
the service area reuses that single definition through a relative `#[path]`
include rather than inventing a second enum.

## Intentionally not implemented

- **Android UI / Activities**: no real `Application`, `Activity`, `Fragment`,
  layout, view binding, GIF/image loading, dialogs, immersion bar or lifecycle.
  `SplashActivity` and `DeepLinkRouterActivity` exist only as names in the boot
  step list / docs, not as types.
- **Android service plumbing**: TheRouter service locator, `Intent`, `Uri`,
  `Bundle`, `Context`, coroutines/`Continuation`, `TaskStackBuilder`, pending
  intent caching and handler registration are abstracted away; only the portable
  ready/navigate contract is kept.
- **Platform side effects**: MMKV/SharedPreferences persistence, locale manager,
  UMeng/Firebase analytics, intro-video network fetch, USB permission flow,
  device-management and the Steam/WinEmu/Landscape module initializers are
  represented as labeled boot steps only.
- **Steam / cloud gaming / remote play**: excluded by rebuild scope.

## Verification

Compiled and tested standalone with `rustc --test` using a temporary harness
that wires the three `mod.rs` files; all unit tests pass (see module test blocks).
