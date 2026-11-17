# Area report — user / auth + guide onboarding (Rust port)

Reference: GameHub Lite v5.1.8 decompiled sources under
`sources/com/xj/user/`, `sources/com/xj/landscape/launcher/ui/guide/`,
`com.xj.landscape.launcher.p000enum.LoginPlatformEnum` and the login/entity
classes in `com.xj.landscape.launcher`. All modules are dependency-free
(std only), plain `pub` structs/enums, with `#[cfg(test)]` unit tests. Layout
follows the project convention: Rust `mod.rs` + sibling `.rs` files at the
mirrored Java package path, to be wired via `#[path]` in `app/src/lib.rs`.

## Files created

| File | Models / source | Contents |
|---|---|---|
| `user/auth_provider.rs` | `LoginPlatformEnum`, guide login buttons | `AuthProvider` enum (Google, Apple, QQ, WeChat, Phone, Email, AliOneKey, Steam), `ALL`, `as_str`/`from_str`, `is_social`, `Display` |
| `user/token_bundle.rs` | `WxAccessToken`, `QQLoginToken`, login token payloads | `TokenBundle` (access/refresh/expiry), `is_expired`, `is_expired_with_leeway`, `remaining_secs`, `is_expired_now`, `never_expires` |
| `user/user_profile.rs` | `com.xj.common.user.UserInfo`, `UserProfileEntity` | `UserProfile` (uid, nickname, avatar_url, provider), `has_avatar`, `Default` |
| `user/auth_state.rs` | guide login / validation / one-key flows | `AuthState` state machine, `AuthEvent`, `AuthFailure`, `AuthEffect`, `transition`, `can_transition`, `AuthStateMachine`, `InvalidTransition` |
| `user/mod.rs` | package root | submodule declarations + re-exports |
| `landscape/launcher/ui/guide/onboarding_step.rs` | `Guide*Activity` / `Guide*Fragment` screens | `OnboardingStep` (25 ordered steps), `ALL`, `position`, `class_name`; `OnboardingChecklist` completion state (`complete`, `uncomplete`, `is_complete`, `next_pending`, `progress`, `reset`) |
| `landscape/launcher/ui/guide/mod.rs` | package root | submodule declaration + re-exports |

## Auth state machine

Canonical cycle: `SignedOut --SignIn(p)--> Authorizing(p) --AuthSucceeded(t)-->
Authenticated(t) --Refresh--> Refreshing --Refreshed(t)--> Authenticated(t)`;
`AuthFailed`/`RefreshFailed` lead to `Failed(reason)` and `Acknowledge` returns
to `SignedOut`. `SignOut` is legal from every state. Invalid events return an
`InvalidTransition` rather than mutating state.

## Sources consulted

- `sources/com/xj/user/` — `UserAvatarView`, `BR`, `R` (package presence; no
  auth logic in the decompiled output).
- `sources/com/xj/landscape/launcher/ui/guide/GuideLoginActivity.java` —
  provider buttons (`googleLoginBtn`, `appleLoginBtn`, `wxLoginBtn`,
  `phoneLoginBtn`), Firebase Google/Apple, QQ/WeChat, `OneKeyAliHelper`,
  e-mail/phone login intents.
- `sources/com/xj/landscape/launcher/p000enum/LoginPlatformEnum.java` —
  `Google, Apple, QQ, WeiXin`.
- `sources/com/xj/landscape/launcher/data/requestparam/ThirdLoginRequestParam.java`
  — Firebase provider ids `google.com` / `apple.com`, uid/email/nickname.
- `sources/com/xj/landscape/launcher/data/model/entity/WxAccessToken.java`,
  `QQLoginToken.java` — token data shapes.
- `sources/com/xj/common/user/UserInfo.java`,
  `sources/com/xj/landscape/launcher/data/model/entity/UserProfileEntity.java`
  — `uid`, `nickname`, `avatar` profile fields.
- `sources/com/xj/landscape/launcher/ui/guide/` (all `Guide*` files) — the
  ordered screen list for the onboarding checklist.

## Intentionally not implemented (out of scope)

- **SDK / network calls**: no FirebaseAuth, Google/Apple sign-in intents,
  QQ/WeChat SDK or `QQLoginToken`/`WxAccessToken` HTTP exchange,
  `OneKeyAliHelper`, Steam linking, token fetch/refresh requests, `Post`/OkHttp
  coroutine code or any I/O. Providers are identity tags only.
- **Android UI**: no `Activity`, `Fragment`, layout/XML, view binding
  (`LlauncherActivityGuideLoginBinding`), `LoginTypeView`, dialogs, permission
  dialogs, focus/selection state or lifecycle. Guide screens are enum names
  with class names, not types.
- **Persistence / platform**: no MMKV/SharedPreferences storage of tokens or
  profile, no `Context`, `Intent`, `Bundle`, `Resources`, coroutines or
  locale/analytics side effects.
- **Protocol details**: wire formats, signature/sign_utils, encryption and
  API response parsing for auth endpoints are excluded by rebuild scope.

## Verification

Compiled and tested standalone with `rustc --test` using a temporary harness
that declares the two `mod.rs` files with `#[path]`; all unit tests pass.
