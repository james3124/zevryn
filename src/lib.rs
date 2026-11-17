//! Zevryn — Rust reimplementation of GameHub Lite v5.1.8.
//!
//! Every module sits at the path of the decompiled source it replaces.
//! The tree under `app/` mirrors the reference structure.

// --- com/xj/app + deeplink + navigation ---
#[path = "../sources/com/xj/app/mod.rs"]
pub mod app_boot;

#[path = "../sources/com/xj/common/deeplink/mod.rs"]
pub mod deeplink;

#[path = "../sources/com/xj/common/service/mod.rs"]
pub mod nav_service;

// --- API client + DTOs ---
#[path = "../sources/com/xj/common/data/mod.rs"]
pub mod data;

#[path = "../sources/com/xj/common/http/mod.rs"]
pub mod http_api;

// --- storage: Room schema, entities, DAO shapes ---
#[path = "../sources/com/xj/common/data/db/mod.rs"]
pub mod db;

#[path = "../sources/com/xj/common/data/table/mod.rs"]
pub mod tables;

#[path = "../sources/com/xj/common/data/dao/mod.rs"]
pub mod dao;

// --- dashboard UI model ---
#[path = "../sources/com/xj/landscape/launcher/ui/main/mod.rs"]
pub mod dashboard;

// --- input mapping + keymap ---
#[path = "../sources/com/xj/mapping/mod.rs"]
pub mod mapping;

#[path = "../sources/com/xiaoji/sdk/mod.rs"]
pub mod xiaoji_sdk;

// --- injection payloads ---
#[path = "../sources/com/xiaoji/inject/mod.rs"]
pub mod inject;

// --- emulator exec model ---
#[path = "../sources/com/winemu/core/mod.rs"]
pub mod winemu_core;

#[path = "../sources/defpackage/mod.rs"]
pub mod defpackage;

// --- resources / localization ---
#[path = "../sources/com/xj/language/mod.rs"]
pub mod language;

// --- device / OTA / download ---
#[path = "../sources/com/xj/ota/mod.rs"]
pub mod ota;

#[path = "../sources/com/xj/apk/mod.rs"]
pub mod apk;

#[path = "../sources/com/xj/devicemanagement/mod.rs"]
pub mod devicemanagement;

#[path = "../sources/com/krly/mod.rs"]
pub mod krly;

// --- download / archive pipeline ---
#[path = "../sources/com/xj/common/download/mod.rs"]
pub mod download;

// --- settings / preferences ---
#[path = "../sources/com/xj/common/preference/mod.rs"]
pub mod preference;

// --- auth & onboarding ---
#[path = "../sources/com/xj/user/mod.rs"]
pub mod user;

#[path = "../sources/com/xj/landscape/launcher/ui/guide/mod.rs"]
pub mod onboarding;

// --- routing registry ---
#[path = "../sources/com/therouter/mod.rs"]
pub mod therouter;

#[path = "../sources/com/xj/common/router/mod.rs"]
pub mod app_router;

// --- gamepad / device protocols ---
#[path = "../sources/com/xj/device/ble/mod.rs"]
pub mod gamepad;

// --- launcher: focus engine, events, viewmodels, game detail ---
#[path = "../sources/com/xj/common/view/focus/mod.rs"]
pub mod focus_engine;

#[path = "../sources/com/xj/landscape/launcher/event/mod.rs"]
pub mod launcher_event;

#[path = "../sources/com/xj/landscape/launcher/vm/mod.rs"]
pub mod launcher_vm;

#[path = "../sources/com/xj/landscape/launcher/ui/gamedetail/mod.rs"]
pub mod game_detail;

// --- offline support ---
#[path = "../sources/com/xj/common/offline/mod.rs"]
pub mod offline;

// --- game settings: item models, selection, backup, env params ---
#[path = "../sources/com/xj/winemu/settings/models/mod.rs"]
pub mod game_settings;

#[path = "../sources/com/xj/winemu/settings/operations/mod.rs"]
pub mod settings_operations;

#[path = "../sources/com/xj/winemu/settings/backup/mod.rs"]
pub mod settings_backup;

#[path = "../sources/com/xj/winemu/settings/env/mod.rs"]
pub mod settings_env;

#[path = "../sources/com/xj/common/data/gameinfo/mod.rs"]
pub mod gameinfo;

// --- import game -> built-in storage ---
#[path = "../sources/com/xj/game/entity/mod.rs"]
pub mod game_entity;

#[path = "../sources/com/xj/game/repository/mod.rs"]
pub mod game_repository;

#[path = "../sources/com/xj/winemu/utils/mod.rs"]
pub mod import_pipeline;

// --- uninstall / remove game ---
#[path = "../sources/com/xj/game/uninstall/mod.rs"]
pub mod uninstall;

// --- game library UI state ---
#[path = "../sources/com/xj/game/ui/game_library/mod.rs"]
pub mod game_library_ui;

// --- third-party / native inventory ---
#[path = "../sources/org/compression.rs"]
pub mod compression;

#[path = "../sources/org/crypto.rs"]
pub mod crypto;

#[path = "../apktool/lib/native.rs"]
pub mod native_libs;

#[path = "../apktool/assets/config.rs"]
pub mod aria_config;
