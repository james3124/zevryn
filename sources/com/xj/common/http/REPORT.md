# API-layer reimplementation report (GameHub Lite v5.1.8)

Scope: portable logic from the decompiled API layer. Read-only reference:
`reference/gamehub-lite-v518-decompiled/gamehub-lite-v518-decompiled/sources/`.
Only `std` is used; no external crates.

## Files written

| File | Models |
| --- | --- |
| `common/http/mod.rs` | module root / public re-exports |
| `common/http/api_config.rs` | `ApiConfig`, `BASE_URL`, legacy hosts, endpoint paths |
| `common/http/client_params.rs` | `ClientParams::value()` — the `|`-joined `clientparams` string |
| `common/http/sign_utils.rs` | `SignUtils` canonical signing + pure-Rust MD5 |
| `common/http/signed_request.rs` | `SignedRequest` / `HttpMethod` — interceptor request-rewrite shape |
| `common/data/mod.rs` | data DTO module root |
| `common/data/json.rs` | local copy of the crate's dependency-free JSON reader/writer |
| `common/data/game_library_table.rs` | `GameLibraryTable` (Room entity DTO) |
| `common/data/game_detail_entity.rs` | `GameDetailEntity` (`@SerializedName` fields + predicates) |
| `common/data/common_download_task_state.rs` | `CommonDownloadTaskState` enum + `type` codes |
| `landscape/launcher/data/mod.rs` | launcher data module root (re-exports the http model) |
| `landscape/launcher/data/repository.rs` | `RepoEndpoint` catalogue + `build_repo_request` |

## Signing caveat

Both `SignUtils` copies (`com.xj.common.http` and `com.xj.adb.wifiui.http`)
are identical. The algorithm is:

1. Collect request params into a `TreeMap` (sorted by key).
2. Join entries as `key=value` separated by `&`.
3. Replace every literal `null` substring with `""`.
4. Append `&all-egg-shell-y7ZatUDk`.
5. MD5 the string and lowercase the hex digest.

A real, self-contained MD5 is implemented in `sign_utils.rs` (no crypto crate),
so signatures are byte-for-byte compatible with the app. `sign` is computed
**before** the `sign` key is inserted into the body, and the legacy adb
interceptor skips signing for exactly `https://clientgsw.vgabc.com/clientapi/`.

Notes / limitations:

- `clientparams` device values are caller-supplied (the Java version reads
  `Build.MODEL`, screen size, locale, GPU/CPU from Android APIs). The *shape*
  (19 ordered, `|`-joined fields) is preserved.
- The deprecated `com.xj.common.http.TokenInterceptor` computes its signature
  from `JSONObject.toString()` plus `uid`/`session`/`version`/`language` and
  adds `format=JSON`; it is not the active interceptor (the HTTP config
  registers `EggGameTokenInterceptor`) and is not modeled, apart from the
  `SignedRequest::with_session_fields` hook.
- Signing keys/algorithm are reconstructed from decompiled code only; no
  live-server verification was performed.

## What was intentionally NOT implemented

- **OkHttp transport**: no client builder, timeouts, cache, cookie jar,
  `LogRecordInterceptor`, `RemoveExtraSlashInterceptor`, `Chucker`, Gson
  converter, error handler or socket-timeout wrapping.
- **Coroutines / Android scope**: `*Repo` coroutine bodies, `Flow`,
  `AndroidScope`/`NetCoroutineScope`, ViewModel callbacks, `Result` handling.
- **Real HTTP I/O**: there is no network layer in this dependency-free crate;
  `SignedRequest` and `build_repo_request` only model request construction.
- **Nested DTO expansion**: nested `GameDetailEntity` objects
  (`card_line_data`, `friend_list`, startup/compatibility params, videos,
  recommendations) are retained as raw JSON `Value` nodes rather than
  expanded into typed structs. `CommonDownloadTask` / `GameDetailDownloadInfo`
  are out of scope (only the `CommonDownloadTaskState` enum is modeled).
- **Room persistence**: `GameLibraryTable` is a parseable DTO only; no SQLite
 /`androidx.room` behavior.
- **`UxHttpConfig`, upload/download repositories, upload multipart bodies**:
  out of scope for this task.
