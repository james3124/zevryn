# entity area — `com.xj.game.entity` (GameHub Lite v5.1.8)

See the full report at `../repository/REPORT.md` (the repository area is the
primary deliverable and contains the complete mapping table, game-type codes,
and out-of-scope notes).

Files in this area:

- `mod.rs` — module root, re-exports all public types.
- `game_type.rs` — `GameType` enum with exact codes `0/1/2`.
- `steam_game_info.rs` — dependency-free `SteamGameInfo` stand-in.
- `imported_game.rs` — `ImportedGame` mirror of `ImportedGameEntity`.
- `storage_entity.rs` — field-compatible `GameLibraryTable` mirror (self-contained).
- `conversion.rs` — `From<ImportedGame> for GameLibraryTable` + `LAUNCH_TYPE_PC_EMULATOR`.
