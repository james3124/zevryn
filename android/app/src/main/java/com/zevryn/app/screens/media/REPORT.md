# Media screens — Zevryn Android launcher

Scope: offline-first, read-only/local demo screens grounded in the decompiled
GameHub Lite reference (album detail, notification list, feedback), added
without touching Rust sources, `MainActivity`, `build.gradle`, CI, or any
other agent's files. No account, cloud, telemetry or native/JNI calls.

## Kotlin (package `com.zevryn.app.screens.media`)

- **`AlbumFragment.kt`** — `RecyclerView` grid (3 columns) of album/screenshot
  items. Each card shows a colored placeholder banner (no image decoding or
  network) plus a title. Clicking a card shows a demo toast. Demo data is
  hardcoded album titles + hex colors; `Color.parseColor` is guarded so a bad
  color can never crash the screen.
- **`NotificationFragment.kt`** — single-column `RecyclerView` of local demo
  notifications (title, message, time). Clicking an unread notification marks
  it read (unread dot hidden, row dimmed, `notifyItemChanged`) and shows a
  "Marked as read" toast.
- **`FeedbackFragment.kt`** — `EditText` + "Send" button. Saves the text to
  `SharedPreferences` (`zevryn_media_feedback`) and shows the toast
  "Feedback saved locally". No network.

All three use `com.zevryn.app.FocusHelper` so D-pad focus walks the grid
(album) or list (notifications, feedback). All views are `focusable` with
`focusableInTouchMode`.

## Layouts (`res/layout/`, all `media_` prefixed)

- `media_album.xml` — header, hint, `albumRecycler`.
- `media_item_album.xml` — card with `albumBanner` (`@drawable/banner_background`,
  recolored at runtime) + `albumTitle`.
- `media_notification.xml` — header, hint, `notificationRecycler`.
- `media_item_notification.xml` — card with `notifUnread` dot, `notifTitle`,
  `notifTime`, `notifMessage`.
- `media_feedback.xml` — header, hint, `feedbackInput`, `feedbackSend`.

Cards reuse existing `@drawable/card_background` and `@drawable/focus_border`
selectors for the pressed/focused gamepad states.

## Strings (`res/values/strings.xml`, appended only)

Only new `media_*` names were added (tabs, headers/hints, demo album titles,
demo notifications, feedback labels, toasts). No existing string was modified.
Tab-label strings `media_tab_album`, `media_tab_notification`,
`media_tab_feedback` are provided for the parent session's wiring.

All `@color` / `@drawable` references resolve against existing resources; no
new colors or drawables were introduced.

## Suggested wiring (parent session)

Instantiate `AlbumFragment()`, `NotificationFragment()` and
`FeedbackFragment()` from `com.zevryn.app.screens.media` in `MainActivity`'s
pager/tabs, using the provided `media_tab_*` strings as labels.
