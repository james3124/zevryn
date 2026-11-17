# `com.winemu.core.regedit` — Rust port report

Dependency-free Rust model of the portable logic in GameHub Lite v5.1.8's
`com.winemu.core.regedit` package. Uses only `std`.

## Files written (all inside this write area)

| File | Rust API | Reference source |
| --- | --- | --- |
| `mod.rs` | package root + `pub use` re-exports | package layout / `PLAN.md` convention |
| `architecture.rs` | `Architecture { Unknown, Win32, Win64 }`, `parse`, `as_str`, `pointer_width` | `Architecture.java` (`UNKNOWN`, `WIN32`, `WIN64`), `RegistryParser.h` (`#arch=`) |
| `value.rs` | `RegValue` enum: `String`, `Dword(u32)`, `Qword(u64)`, `Binary(Vec<u8>)`, `MultiString(Vec<String>)`, `None`; `type_code`, `to_le_bytes`, `Display` | `RegistryValue.java` (StringValue=1, ExpandStringValue=2, BinaryValue=3, DwordValue=4, MultiStringValue=7, QwordValue=11; `a()` bytes, `c()` type) |
| `key.rs` | `RegKey { path, values, subkeys }`, `split_reg_path`, `canonical_path`; `RegHive` with `lookup`, `ensure_key`, `insert_value`, `get_value`, `delete_value`, `delete_key`, `keys` | `RegistryKey.java` (`b`/`a` get-or-create, `g` lookup, `y`/`z` value insert, `f` value delete, `e` key delete, `Companion.a()` = `\REGISTRY` root) |
| `patch.rs` | `RegChange` (six variants), `KeyPropertyChange`, `RegPatch` builders, `order_changes`, `apply`/`apply_with_options`, `PatchOptions`, `PatchFailure`, `RegResult` report | `RegistryPatcher.java` (PatchOptions/PatchResult/PatchFailure), `RegistryComparator.java` (RegistryChange/KeyPropertyChange), the eight `RegistryPatcher$orderChanges$$inlined$*` comparator helper classes |
| `parser.rs` | `RegParseError`, `RegLoadResult`, `parse_reg`, `parse_hive`, `write_reg` | `RegistryParser.java` (`k` header, `h` arch, `i` keys, `v` names, `l` values, `c` types, `q` hex bytes, `r`/`u` escapes), `RegistryWriter.java` (text emission half) |

`#[cfg(test)]` tests (19 total, all passing) cover tree insert/lookup/delete,
recursive vs non-recursive key deletion, path helpers, comparator ordering
(shallow-first key adds, deep-first key deletes, stable group order), patch
apply with success and failure reports plus option handling, no-op key-property
changes, `.reg` value/escape/multi-string/continuation parsing, parse errors
with line numbers, and a full parse → serialize → parse round-trip over all
modeled value types.

## Modeling decisions

* **Value names** live in `RegKey.values: BTreeMap<String, RegValue>` rather
  than inside `RegValue` (the Java subclasses store a `name` field, but the
  key's `TreeMap` is the authoritative name→value mapping). `""` is the
  default value written as `@=` in `.reg` files.
* **`BTreeMap`** reproduces the deterministic sorted iteration of the Java
  `TreeMap` value/child maps. Lookup uses exact names; the case-insensitive
  `String.CASE_INSENSITIVE_ORDER` comparator used only for Wine's raw-file
  lookup is not modeled.
* **`ExpandStringValue` (REG_EXPAND_SZ)** is folded into `RegValue::String`;
  `parse_reg` decodes `hex(2):` into a plain string. The type code reported is
  1 for these values.
* **Comparator ordering reconstruction.** The outer bodies of
  `RegistryPatcher` and `RegistryComparator` were stripped in the decompiled
  reference; only the data classes and the eight inline comparator helper
  classes survive. Those helpers define: `KeyAdded` by path depth ascending
  then index, `KeyDeleted` by depth descending then index, and the other four
  change kinds by index. `order_changes()` reproduces those comparators and
  concatenates groups in a deterministic application order (delete values →
  delete keys → add keys → add values → modify values → modify key
  properties). The group order is inferred, but the within-group comparators
  are taken directly from the surviving classes.
* **`RegistryEditor`, `RegistryResult`, `RegistryDsl*`, `RegistryWriter`** are
  thin pass-through/wrapper types in the reference. Their portable behavior is
  exposed directly as `parse_reg`, `write_reg`, `RegLoadResult` and
  `RegPatch::apply`; no separate wrapper structs were added.
* `KeyModified` changes are reported as applied but do not mutate metadata:
  `className`, modification time and the `isSymlink` / `isVolatile` / `isDirty`
  flags of `RegistryKey` are intentionally not part of the portable model.

## Out of scope

* Real Windows/Wine registry access — no reading or writing of actual registry
  hives, no `RegOpenKeyEx`/`Nt*`/`advapi32` calls, no file I/O. The parser works
  on `&str` and the writer returns `String`; opening files is the caller's job.
* Android/JNI integration — no `JNIEnv`, `Context`, `AssetManager`, Wine
  prefix paths, native libraries (`libwinemu.so`, `libvfs.so`) or process
  spawning.
* Binary hive formats, registry transactions, ACL/security descriptors,
  symlink resolution and the exact Wine timestamp metadata.
* Kotlin DSL marker annotations / lambda-scoped builders — the Rust API uses
  plain builder methods instead of a DSL.

## Verification

Compiled standalone with:

```
rustc --test --edition 2021 <harness declaring #[path=.../regedit/mod.rs] mod regedit;>
```

Result: **19 passed; 0 failed**. No Cargo.toml, `app/src/lib.rs`, or any other
agent's directory was modified.
