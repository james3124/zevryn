# Added screens — Zevryn Android launcher

Scope: offline-first settings and a simplified device/controller screen, added
without touching Rust sources, `MainActivity`, `build.gradle`, or CI. The parent
session owns wiring these into `MainActivity`'s `ViewPager2` tabs.

## Kotlin (package `com.zevryn.app.screens`)

- **`SettingsFragment.kt`** — Settings tab. A `RecyclerView` of rows backed by
  `fragment_settings.xml` / `item_setting_row.xml`:
  - Offline Mode toggle (`SwitchCompat`, persisted in SharedPreferences)
  - Import Game (demo toast)
  - Gamepad / Key Mapping (demo toast)
  - About — shows the static string `Zevryn 0.1.0 / Rust core offline-capable`
    (deliberately no native calls; guarded JNI accessors live only on
    `MainActivity` and the external functions are private).
  - No account, cloud, or telemetry entries.
- **`DeviceFragment.kt`** — Devices tab. Placeholder device rows (name + status)
  plus a "Scan for devices" button that shows a `Scanning...` toast. Inspired by
  a device manager / controller list, but performs no real discovery.

Both fragments use `com.zevryn.app.FocusHelper` with a single column so D-pad
focus walks up/down the lists.

## Layouts (`res/layout/`)

- `fragment_settings.xml` — header, hint, `settingsRecycler`.
- `item_setting_row.xml` — title + subtitle + optional `SwitchCompat`.
- `fragment_device.xml` — header, hint, `scanButton`, `deviceRecycler`.
- `item_device.xml` — device icon (`@drawable/ic_device`), name, status.

All widgets use existing AppCompat/Material/RecyclerView classes from the
current dependencies, and all `@color` / `@drawable` references already exist.

## Strings (`res/values/strings.xml`, appended only)

Added `tab_settings`, `tab_device`, `settings_*`, `setting_*`, and `device_*`
strings. No existing string was modified. No new colors or drawables required.

## Suggested wiring (parent session)

Add two tabs in `MainActivity` (importing the two fragments and using
`R.string.tab_settings` / `R.string.tab_device`), e.g. `getItemCount() = 4` and
map positions 2/3 to `SettingsFragment()` / `DeviceFragment()`.
