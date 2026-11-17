# Zevryn Android launcher UI — build report

The plain `TextView` shell has been replaced with a real, landscape, gamepad-first
launcher UI built entirely in the Android shell directories. No Rust files,
Cargo files, or CI workflows were modified.

## JNI functions used

Declared as **instance methods** on `com.zevryn.app.MainActivity` so the Rust
`#[no_mangle]` symbols `Java_com_zevryn_app_MainActivity_*` resolve correctly:

| Kotlin declaration | Rust export | Purpose |
| --- | --- | --- |
| `private external fun coreVersionCode(): Int` | `Java_com_zevryn_app_MainActivity_coreVersionCode` | Core version shown in dashboard header |
| `private external fun coreModuleCount(): Int` | `Java_com_zevryn_app_MainActivity_coreModuleCount` | Module count shown in dashboard header |
| `private external fun nativeDashboardCards(): String` | `Java_com_zevryn_app_MainActivity_nativeDashboardCards` | JSON array `{title, kind, color}` -> dashboard card grid |
| `private external fun nativeGameLibrary(): String` | `Java_com_zevryn_app_MainActivity_nativeGameLibrary` | JSON array `{id, title, kind}` -> game library grid |

`org.json.JSONArray` (built into Android) parses both JSON strings — no new
JSON dependency. Every native call is wrapped in `try/catch` with `Log.i/e`
under tag `Zevryn`; failures fall back to `-1` / `"[]"` so the app always opens.

## Kotlin files

- `app/src/main/java/com/zevryn/app/MainActivity.kt` — `AppCompatActivity`, loads
  `libzevryn.so` in a companion `init`, hosts `TabLayout` + `ViewPager2` with two
  tabs (Dashboard, Instant Play), exposes guarded accessors to fragments.
- `app/src/main/java/com/zevryn/app/DashboardFragment.kt` — parses
  `nativeDashboardCards()`, populates the `my_container_view` card grid.
- `app/src/main/java/com/zevryn/app/LibraryFragment.kt` — parses
  `nativeGameLibrary()`, sort/filter spinners drive the game grid.
- `app/src/main/java/com/zevryn/app/DashboardCardAdapter.kt` — card ViewHolder,
  hex colour banner (`Color.parseColor`), short-click Toast, focus wiring.
- `app/src/main/java/com/zevryn/app/LibraryGameAdapter.kt` — game row ViewHolder,
  kind badge colour, short-click Toast, focus wiring.
- `app/src/main/java/com/zevryn/app/FocusHelper.kt` — assigns stable ids and
  `setNextFocusLeft/Right/Up/Down` for grid D-pad navigation.

## Layouts (`res/layout/`)

- `activity_main.xml` — mirrors reference ids: `main_tab_view` (top tab bar),
  `ll_right_top_status` with `right_user_avatar_view`, `iv_search`,
  `iv_downloading`, `iv_device_online`, `iv_signal_level`, `iv_battery_level`,
  `tv_battery_percent`, `tv_system_time` (TextClock HH:mm), `fl_open_nav` /
  `iv_open_nav`, `iv_tips_lb` / `iv_tips_rb`, and `page_container` (ViewPager2).
- `fragment_dashboard.xml` — `iv_my_page_background` + `my_container_view` grid.
- `fragment_library.xml` — sort/filter Spinner bar + `libraryRecycler`.
- `item_game_card.xml` — coloured banner + kind chip + title, focusable.
- `item_library_game.xml` — kind badge + title + kind, focusable.

## Values & drawables

- `values/strings.xml`, `values/colors.xml`, `values/styles.xml`
  (`Theme.Zevryn` parent `Theme.MaterialComponents.NoActionBar`, dark landscape).
- `drawable/card_background.xml` and `drawable/focus_border.xml` are shape
  selectors providing the visible focus ring; plus banner/badge/chip/spinner/
  background shapes and vector status icons (`ic_*`).

## Gradle

`app/build.gradle` keeps `applicationId com.zevryn.app`, `compileSdk 35`,
`minSdk 26`, Java/Kotlin 17, and the `arm64-v8a` ABI split. Added dependencies:
appcompat 1.7.0, material 1.12.0, recyclerview 1.3.2, viewpager2 1.1.0,
fragment-ktx 1.8.5. `libzevryn.so` is still supplied via `jniLibs/arm64-v8a/`.
