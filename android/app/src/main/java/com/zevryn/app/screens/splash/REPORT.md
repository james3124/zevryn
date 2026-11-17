# Splash screens — Zevryn Android launcher

Scope: launch/permission-info UI only. No Rust, `MainActivity`, manifest,
`build.gradle` or CI changes. No account, login, cloud or telemetry.

## Kotlin (package `com.zevryn.app.screens.splash`)

- **`SplashActivity.kt`** — full-screen launch activity. Inflates
  `splash_activity`, animates a determinate `ProgressBar` from 0 to 100 over
  ~1.2 s (driven by a main-looper `Handler` with `postDelayed`), then starts
  `MainActivity` and calls `finish()` so the splash is not kept in the back
  stack. Crash-safe: progress updates and `startActivity` are wrapped in
  `try/catch`, navigation is single-shot via a `navigated` flag, and pending
  callbacks are removed in `onDestroy()`. Uses only existing deps (AppCompat,
  Android OS). **Not** registered in the manifest as launcher — the parent
  session owns that; only the class is provided.
- **`PermissionNoteFragment.kt`** — optional info screen (no real permission
  requests). Inflates `splash_permission` and shows a confirmation toast when
  the "Got it" button is tapped. Extends `androidx.fragment.app.Fragment`.

## Layouts (`res/layout/`, `splash_` prefix only)

- `splash_activity.xml` — centered text-based "Zevryn" logo (`@string/app_name`)
  + tagline + horizontal progress bar; dark `windowBackground`.
- `splash_permission.xml` — centered Permissions title, explanatory note and
  acknowledgement button.

## R references (all resolve to existing or newly added resources)

- Layouts/IDs added here: `R.layout.splash_activity`, `R.layout.splash_permission`,
  `R.id.splashProgress`, `R.id.splashTitle`, `R.id.splashTagline`,
  `R.id.permissionAck`.
- Existing strings reused: `@string/app_name`, `@string/settings_hint`.
- Existing colors reused: `@color/windowBackground`, `@color/textPrimary`,
  `@color/textSecondary`.
- No new drawables, colors, strings or dependencies required.
