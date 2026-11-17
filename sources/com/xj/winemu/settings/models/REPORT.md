# Area report — settings models (Rust port)

Port of the portable logic under
`reference/gamehub-lite-v518-decompiled/gamehub-lite-v518-decompiled/sources/com/xj/winemu/settings/models/`
(the `GroupSetting` / `ChildSetting` settings tree and the `SettingTypeValue`
family), plus `com.xj.winemu.settings.EditType`. Dependency-free (std only),
with `#[cfg(test)]` unit tests.

## Files created

| File | Models | Contents |
|---|---|---|
| `edit_type.rs` | `com.xj.winemu.settings.EditType` | `EditType` enum with the two real constants, `ALL`, `as_str`/`from_name`, `ordinal`/`from_ordinal`, `Display` |
| `icons.rs` | `Icons` | `Icons::LaunchContainer` (Java `launch_container`), `ALL`, `as_str`/`from_name`, `Display` |
| `setting_type_value.rs` | `SettingTypeValue` + 3 payloads | `SettingTypeValue` enum (`Switch`/`Button`/`InnerInput`), `SwitchSettingTypeValue`, `ButtonSettingTypeValue`, `InnerInputSettingTypeValue`, constructors + `kind`/`as_switch` |
| `child_setting.rs` | `ChildSetting` | `key`/`label`/`desc`/`edit_type`/`value`/`default`/`enabled`/`icon`, `new`/`action`, builder setters, `set_value`/`reset`/`is_modified` |
| `group_setting.rs` | `GroupSetting` | `id`/`title`/`icon`/`children`, `new`/`named`, `push`/`insert`, `find`/`find_mut`, `len`/`is_empty` |
| `settings_model.rs` | portable `PcGameSettingsKt` tree | `SettingsModel` with `find`/`find_mut`, `set_value` (in place), `with_value` (returns updated tree), `reset_to_defaults`/`with_defaults`, `groups`, `flatten`, `validate`; free `flatten`/`validate`; `ValidationError` |
| `mod.rs` | package root | submodule declarations + re-exports |

## Exact `EditType` values + source

The task brief suggested values such as `SWITCH` / `BUTTON` / `INPUT` / `JUMP`,
but those strings do **not** occur in `EditType.java`. Following the instruction
to "use only values found", the Rust enum contains exactly the two constants
present in the source, in declaration order:

| Variant | Java constant | Ordinal | Source |
|---|---|---|---|
| `EditType::EditGameVars` | `EditGameVars` | 0 | `sources/com/xj/winemu/settings/EditType.java` line 20; `apktool/smali_classes4/com/xj/winemu/settings/EditType.smali` lines 20, 35–41, 79 |
| `EditType::EditTranslationConfigName` | `EditTranslationConfigName` | 1 | `sources/com/xj/winemu/settings/EditType.java` line 21; `apktool/smali_classes4/com/xj/winemu/settings/EditType.smali` lines 22, 45–51, 81 |

Both the decompiled Java and the smali independently agree that the enum has no
other constants. For context, the SWITCH/JUMP-style names the brief alluded to
belong to *other* types, which are intentionally not merged into `EditType`:

- `sources/com/xj/winemu/bean/SettingType.java` — `Option`, `Switch`, `Input`.
- `PcSettingItemEntity.Companion` constants `TYPE_SWITCH` / `TYPE_JUMP_ITEM`
  used throughout `PcGameSettingsKt.java` and `PcGameSettingDataHelper.java`.
- The three `SettingTypeValue` implementations (switch / button / inner input)
  model the *value payload* of a row, not `EditType`.

## Design notes

- `SettingTypeValue` is an empty `Parcelable` marker interface in Java; it is
  modeled as a Rust enum over the three concrete payloads, which is the natural
  portable equivalent.
- Non-portable payload fields are dropped: `Function1 onSwitchChanged` /
  `onTextChanged`, `ChildSetting`'s `Function1 action` / `Function1 onChanged` /
  `Function0 click`, and `ButtonSettingTypeValue`'s Android `Radii radius`.
  `ButtonSettingTypeValue.action: Option<String>` is a plain-data stand-in for
  the logical button click target.
- The Java `ChildSetting` has **no** `EditType` field. The tree API requested an
  `edit_type` on each child, so it is stored as an attribute defaulting to the
  first declared constant (`EditGameVars`); set it explicitly when porting a
  row that opens the translation-config editor.
- `default` is an `Option<SettingTypeValue>` (the portable default value restored
  by `reset_to_defaults`); this generalizes Java's display-only
  `defaultRightText: String`.
- `set_value` mutates in place and reports success; `with_value` returns the
  updated tree by value, as requested. `with_defaults` is the immutable reset
  counterpart. `flatten` yields all children in group-then-position order;
  `validate` reports empty keys/labels, duplicate keys and value/default kind
  mismatch.

## Out of scope

- Android UI / viewholders: `PcGameSettingsActivity`, the
  `settings/holder/*ViewHolder` classes, `CommPcSettingItemFragment`,
  `TranslationConfigSettingsAdapter`, `DialogSettingListItemEntity`,
  `SelectAndSingleInputDialog`, `ImageView.setImage`/`Radii` view rendering.
- Android framework plumbing: `Parcelable`/`@Parcelize`/`CREATOR`, `Function0`/
  `Function1`/`Function2` callbacks, `@Metadata`, `EnumEntries`, `Intrinsics`,
  Kotlin DSL lambdas, `R.string.*` resources and `Utils.a()` localization.
- Data/IO layer behind the list: `PcGameSettingDataHelper`,
  `PcGameSettingOperations`, `PcSettingDataEntity`, `WineActivityData`,
  `IWinEmuService`, file/MD5 lookups (`getPcGameSettingValue`), channels and
  coroutine `Continuation` state machines.
- The many `PcGameSettingsKt$*${1}...` synthetic callback classes (they encode
  Android click/dialog behaviour, not portable data).
