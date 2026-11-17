# Router reimplementation — report

Rust, `std` only. No dependency added to `Cargo.toml`; `app/src/lib.rs` and other
agents' directories were not touched.

## Files written

### `app/sources/com/therouter/` (framework-level model)
| File | Contents |
| --- | --- |
| `mod.rs` | Re-exports `Route`, `extract_placeholders`, `RouteTable`, `ResolvedRoute`, `ServiceProvider`, `default_provider`. |
| `route.rs` | `pub struct Route { path, target, params }`. `Route::new` auto-extracts `{name}` placeholders from the path into `params`; `with_params` adds declared names. |
| `route_table.rs` | `pub struct RouteTable` with `register` / `register_all` / `resolve`. `{name}` placeholder matching is per path segment; query strings are split off before matching; exact patterns win over placeholder patterns. `ResolvedRoute` exposes captured + query params via `param()`. |
| `service_provider.rs` | `pub struct ServiceProvider` mapping interface name -> implementation name (register/get/len). `default_provider()` preloads all 17 bindings found in the generated service files. |

### `app/sources/com/xj/common/router/` (app-specific table + request builder)
| File | Contents |
| --- | --- |
| `mod.rs` | Re-exports everything above. Includes the framework router by relative `#[path = "../../../therouter/mod.rs"]` so this area is self-contained and does not depend on how the crate root names its modules. |
| `common_router_table.rs` | Static `ROUTES: &[RouteDef]` (35 `(path, target)` pairs, `&'static str`), `RouteDef`, `build_route_table() -> RouteTable`, `THEROUTER_APT_VERSION = "1.3.0-rc4"`. |
| `route_request.rs` | Locally re-declared 7-value `DeepLinkPage` enum (values/wire strings mirror `GhDeepLinkPageType`) + `route_path_for(page)` + `RouteRequest` builder (`new`, `with_path`, `param`, `url`). Self-contained by design. |
| `page_router_utils.rs` | Route-path helpers mirroring the portable choices in `PageRouterUtils`: download manager, game library, winemu env settings, and `route_for_page`. |

## Exact generated route-table filenames read

From
`reference/gamehub-lite-v518-decompiled/gamehub-lite-v518-decompiled/sources/a/`:

- `RouterMap__TheRouter__1270983578.java` — winemu (env layer detail, env setting, download manager, PC game settings) — **4 routes**
- `RouterMap__TheRouter__1735410217.java` — steam login / change account — **2 routes**
- `RouterMap__TheRouter__1913622041.java` — game library — **1 route**
- `RouterMap__TheRouter__1938593753.java` — landscape launcher (user info, search, common fragment, mobile game, main activity, guide, news/game detail, feedback, product doc, web) — **12 routes**
- `RouterMap__TheRouter__2115427588.java` — cloud (setting, launcher, exchange) — **3 routes**
- `RouterMap__TheRouter__2135221615.java` — cloud game pay — **1 route**
- `RouterMap__TheRouter__318161087.java` — device setting / gamepad — **5 routes**
- `RouterMap__TheRouter__397122787.java` — OTA activities — **7 routes**

Total: **35 routes**, all reproduced verbatim in `ROUTES` (including the required
winemu / gamelibrary / env / settings paths).

Service bindings were taken from these generated files (all 11 read):
`ServiceProvider__TheRouter__1344554815.java`, `2000493684.java`, `2056161029.java`,
`549289766.java`, `573075289.java`, `634971008.java`, `742311270.java`,
`892010023.java`, `911463050.java`, `923475518.java`, `969355976.java`.

Deep-link page values came from
`sources/com/xj/common/deeplink/GhDeepLinkPageType.java` (7 enum constants) and
route usage from `GameHubDeepLinkManager.java`, `DefaultDeepLinkHandler.java`,
and `sources/com/xj/common/router/PageRouterUtils.java`.

## Out of scope (deliberately not modeled)

- **Annotation processing / APT code generation** — the `@Route`/`@ServiceProvider`
  annotations and the kymjs APT machinery that emits the `RouterMap__*`/`ServiceProvider__*`
  classes. Only its generated *output* (path/target/binding constants) is captured.
- **Android intent / navigation plumbing** — `Navigator`, `TheRouter.d/e`,
  `Intent`, `Context`, `Activity`, `Bundle`, `TaskStackBuilder`, coroutine
  dispatchers, and `RegexpKeyedMap`'s actual `Pattern` compilation. We reproduce
  matching semantics with plain string/segment logic instead.
- Flow-task initialization (`Digraph`, `FlowTaskRunnable`) declared alongside
  the service providers; only the interface -> implementation bindings are kept.
- `Interceptor.interception` argument-count checks and object instantiation
  (all GameHub routes use zero-arg bindings).

## Tests / verification

`#[cfg(test)]` tests cover: resolve with `{name}` placeholder params plus query
params, exact-vs-placeholder precedence, service lookup and replacement, the
static route table (35 entries, winemu env route present), and the 7 page wire
values. Verified with `cargo test` in a throwaway harness that declares both
module directories: **21 tests passed, 0 failed**.

## Crate-root wiring note (for the parent session)

`app/src/lib.rs` was not modified. To expose these modules, declare e.g.:

```rust
#[path = "../sources/com/therouter/mod.rs"]
pub mod therouter;

#[path = "../sources/com/xj/common/router/mod.rs"]
pub mod router;
```

The `common/router` module works on its own (it includes `therouter` via a
relative path), so declaring it alone is also sufficient.
