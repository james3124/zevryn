//! Portable UI state for the game-detail screen, modelled on the `LiveData`
//! flows in `com.xj.landscape.launcher.vm.GameDetailVM` (detail fetch success /
//! failure drives the screen between loading and loaded states).
//!
//! No Android `LifecycleOwner`, coroutine scopes or `UnPeekLiveData` here —
//! only pure state and transitions.

use super::start_type::{GameStartType, code};

/// Minimal portable game-detail payload. Mirrors the fields of
/// `com.xj.common.data.gameinfo.GameDetailEntity` that affect launching.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GameDetail {
    /// `GameDetailEntity.id`.
    pub game_id: i64,
    /// `GameDetailEntity.valid_id`.
    pub valid_id: String,
    /// Display name (`GameDetailEntity.name` / `game_name`).
    pub name: String,
    /// Raw `start_type` / `currentOpenType` value, if known.
    pub start_type_code: Option<i32>,
}

impl GameDetail {
    /// Resolve the raw code into a [`GameStartType`] without inventing codes.
    pub fn start_type(&self) -> Option<GameStartType> {
        self.start_type_code.and_then(GameStartType::from_code)
    }

    /// True when this detail describes the decompiled "PC emulator" game type
    /// (`1403` or `1407`).
    pub fn is_pc_emu_game(&self) -> bool {
        self.start_type_code
            .map(|c| c == code::START_TYPE_PC_EMULATOR || c == code::START_TYPE_STEAM_GAME_BY_PC_EMULATOR)
            .unwrap_or(false)
    }
}

/// State of the game-detail screen.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GameDetailUiState {
    Idle,
    Loading,
    Loaded { detail: GameDetail },
    Error { message: String },
}

impl Default for GameDetailUiState {
    fn default() -> Self {
        Self::Idle
    }
}

/// Events that drive [`GameDetailUiState`] transitions.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GameDetailEvent {
    /// User/app requested a (re)load of the detail.
    Load,
    /// Fetch completed with a detail payload.
    Loaded(GameDetail),
    /// Fetch failed.
    Failed(String),
    /// Return to the initial state.
    Reset,
}

impl GameDetailUiState {
    /// Apply an event, returning the next state. Transitions are total: any
    /// event is valid from any state.
    pub fn apply(self, event: GameDetailEvent) -> Self {
        match event {
            GameDetailEvent::Load => Self::Loading,
            GameDetailEvent::Loaded(detail) => Self::Loaded { detail },
            GameDetailEvent::Failed(message) => Self::Error { message },
            GameDetailEvent::Reset => Self::Idle,
        }
    }

    /// Transition helpers (consuming, for ergonomic chaining).
    pub fn load(self) -> Self {
        self.apply(GameDetailEvent::Load)
    }

    pub fn loaded(self, detail: GameDetail) -> Self {
        self.apply(GameDetailEvent::Loaded(detail))
    }

    pub fn error(self, message: impl Into<String>) -> Self {
        self.apply(GameDetailEvent::Failed(message.into()))
    }

    pub fn reset(self) -> Self {
        self.apply(GameDetailEvent::Reset)
    }

    pub fn is_loading(&self) -> bool {
        matches!(self, Self::Loading)
    }

    pub fn is_loaded(&self) -> bool {
        matches!(self, Self::Loaded { .. })
    }

    /// Borrow the loaded detail, if any.
    pub fn detail(&self) -> Option<&GameDetail> {
        match self {
            Self::Loaded { detail } => Some(detail),
            _ => None,
        }
    }
}

/// A tiny state machine wrapper for callers that prefer event-driven updates.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GameDetailStateMachine {
    pub state: GameDetailUiState,
}

impl GameDetailStateMachine {
    pub fn new() -> Self {
        Self { state: GameDetailUiState::Idle }
    }

    pub fn dispatch(&mut self, event: GameDetailEvent) -> &GameDetailUiState {
        self.state = std::mem::replace(&mut self.state, GameDetailUiState::Idle).apply(event);
        &self.state
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn detail(code: i32) -> GameDetail {
        GameDetail {
            game_id: 7,
            valid_id: "v7".into(),
            name: "Game".into(),
            start_type_code: Some(code),
        }
    }

    #[test]
    fn happy_path_transitions() {
        let s = GameDetailUiState::default();
        assert_eq!(s, GameDetailUiState::Idle);
        let s = s.load();
        assert_eq!(s, GameDetailUiState::Loading);
        assert!(s.is_loading());
        let s = s.loaded(detail(1403));
        assert_eq!(s, GameDetailUiState::Loaded { detail: detail(1403) });
        assert!(s.is_loaded());
        assert_eq!(s.detail().map(|d| d.game_id), Some(7));
        assert_eq!(s.reset(), GameDetailUiState::Idle);
    }

    #[test]
    fn error_and_retry_transitions() {
        let s = GameDetailUiState::Loading.error("network down");
        assert_eq!(s, GameDetailUiState::Error { message: "network down".into() });
        let s = s.load().loaded(detail(1302));
        assert!(s.is_loaded());
    }

    #[test]
    fn detail_resolves_start_type_without_inventing_codes() {
        assert_eq!(detail(1407).start_type(), Some(GameStartType::SteamNative));
        assert_eq!(detail(1403).start_type(), Some(GameStartType::LocalPcEmu));
        assert_eq!(detail(9999).start_type(), None);
        assert!(detail(1403).is_pc_emu_game());
        assert!(detail(1407).is_pc_emu_game());
        assert!(!detail(1302).is_pc_emu_game());
        assert!(!GameDetail::default().is_pc_emu_game());
    }

    #[test]
    fn state_machine_dispatches_events() {
        let mut sm = GameDetailStateMachine::new();
        assert_eq!(sm.dispatch(GameDetailEvent::Load), &GameDetailUiState::Loading);
        assert_eq!(
            sm.dispatch(GameDetailEvent::Loaded(detail(1406))),
            &GameDetailUiState::Loaded { detail: detail(1406) }
        );
        assert_eq!(
            sm.dispatch(GameDetailEvent::Failed("x".into())),
            &GameDetailUiState::Error { message: "x".into() }
        );
    }
}
