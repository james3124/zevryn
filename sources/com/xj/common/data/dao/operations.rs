//! Portable operation model for the Room `@Dao` interfaces of GameHub Lite
//! v5.1.8 (`sources/com/xj/common/data/dao/`).
//!
//! The concrete DAO methods are full of Android/Kotlin `Flow`, `LiveData`,
//! `Continuation` and suspended types, which are not portable. What *is*
//! portable is the kind of persistence operation each method performs (the
//! Room annotation) and the CRUD shape shared across DAOs. Those two aspects
//! are modeled here as enums, with a table mapping each DAO to its operations.

/// The Room write/read annotation category of a DAO method.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DaoAnnotation {
    Insert,
    InsertOrReplace,
    Update,
    Delete,
    Query,
}

impl DaoAnnotation {
    /// Parse an annotation name as written in the decompiled sources.
    pub fn from_annotation(name: &str) -> Option<Self> {
        match name.trim_start_matches('@') {
            "Insert" => Some(Self::Insert),
            "InsertOrReplace" | "Upsert" => Some(Self::InsertOrReplace),
            "Update" => Some(Self::Update),
            "Delete" => Some(Self::Delete),
            "Query" => Some(Self::Query),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Insert => "Insert",
            Self::InsertOrReplace => "InsertOrReplace",
            Self::Update => "Update",
            Self::Delete => "Delete",
            Self::Query => "Query",
        }
    }
}

/// CRUD-shaped operation names that recur across the app's DAO interfaces.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DaoOperation {
    Insert,
    InsertOrReplace,
    Update,
    Delete,
    DeleteAll,
    DeleteById,
    QueryAll,
    QueryById,
    QueryByUser,
    QueryByUid,
    Count,
}

impl DaoOperation {
    /// The Room annotation that backs this operation.
    pub fn annotation(self) -> DaoAnnotation {
        match self {
            Self::Insert => DaoAnnotation::Insert,
            Self::InsertOrReplace => DaoAnnotation::InsertOrReplace,
            Self::Update => DaoAnnotation::Update,
            Self::Delete => DaoAnnotation::Delete,
            // The "Delete*" / "Query*" / "Count" helpers are declared with
            // @Query (annotated SQL) in the decompiled sources.
            Self::DeleteAll | Self::DeleteById => DaoAnnotation::Query,
            Self::QueryAll
            | Self::QueryById
            | Self::QueryByUser
            | Self::QueryByUid
            | Self::Count => DaoAnnotation::Query,
        }
    }
}

/// The DAO interfaces declared in `com.xj.common.data.dao`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DaoKind {
    EmulatorDao,
    GameLibraryDao,
    KernelDao,
    MainTagDao,
    ScreenCaptureDao,
    ScreenRecordDao,
    StarterGameDao,
    SteamAccountDao,
    SteamDownloadDao,
    WinEmuOpRecordDao,
    WinEmuTranslationConfigDao,
}

impl DaoKind {
    pub const ALL: [Self; 11] = [
        Self::EmulatorDao,
        Self::GameLibraryDao,
        Self::KernelDao,
        Self::MainTagDao,
        Self::ScreenCaptureDao,
        Self::ScreenRecordDao,
        Self::StarterGameDao,
        Self::SteamAccountDao,
        Self::SteamDownloadDao,
        Self::WinEmuOpRecordDao,
        Self::WinEmuTranslationConfigDao,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::EmulatorDao => "EmulatorDao",
            Self::GameLibraryDao => "GameLibraryDao",
            Self::KernelDao => "KernelDao",
            Self::MainTagDao => "MainTagDao",
            Self::ScreenCaptureDao => "ScreenCaptureDao",
            Self::ScreenRecordDao => "ScreenRecordDao",
            Self::StarterGameDao => "StarterGameDao",
            Self::SteamAccountDao => "SteamAccountDao",
            Self::SteamDownloadDao => "SteamDownloadDao",
            Self::WinEmuOpRecordDao => "WinEmuOpRecordDao",
            Self::WinEmuTranslationConfigDao => "WinEmuTranslationConfigDao",
        }
    }

    /// Representative portable operations found in this DAO.
    pub fn operations(self) -> &'static [DaoOperation] {
        use DaoOperation as Op;
        match self {
            Self::EmulatorDao => &[Op::Delete, Op::Insert, Op::QueryAll, Op::QueryById, Op::Update],
            Self::GameLibraryDao => &[
                Op::Insert,
                Op::InsertOrReplace,
                Op::QueryAll,
                Op::QueryByUser,
                Op::Update,
            ],
            Self::KernelDao => &[Op::Delete, Op::Insert, Op::QueryAll, Op::Update],
            Self::MainTagDao => &[Op::Delete, Op::Insert, Op::QueryAll, Op::Update],
            Self::ScreenCaptureDao => &[
                Op::Insert,
                Op::QueryAll,
                Op::QueryById,
                Op::Update,
                Op::DeleteById,
            ],
            Self::ScreenRecordDao => &[Op::Insert, Op::QueryAll, Op::QueryByUid, Op::Update],
            Self::StarterGameDao => &[Op::Delete, Op::Insert, Op::QueryAll, Op::QueryById, Op::Update],
            Self::SteamAccountDao => &[Op::Delete, Op::DeleteAll, Op::QueryAll, Op::QueryById],
            Self::SteamDownloadDao => &[
                Op::Insert,
                Op::QueryAll,
                Op::QueryByUid,
                Op::QueryById,
                Op::Update,
            ],
            Self::WinEmuOpRecordDao => &[
                Op::Insert,
                Op::Delete,
                Op::QueryAll,
                Op::QueryByUid,
                Op::Update,
            ],
            Self::WinEmuTranslationConfigDao => &[Op::Insert, Op::QueryAll, Op::QueryById, Op::Count],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_room_annotations() {
        assert_eq!(DaoAnnotation::from_annotation("@Insert"), Some(DaoAnnotation::Insert));
        assert_eq!(DaoAnnotation::from_annotation("Delete"), Some(DaoAnnotation::Delete));
        assert_eq!(DaoAnnotation::from_annotation("@Query"), Some(DaoAnnotation::Query));
        assert_eq!(
            DaoAnnotation::from_annotation("@Upsert"),
            Some(DaoAnnotation::InsertOrReplace)
        );
        assert_eq!(DaoAnnotation::from_annotation("Unknown"), None);
    }

    #[test]
    fn all_daos_have_at_least_one_operation() {
        assert_eq!(DaoKind::ALL.len(), 11);
        for dao in DaoKind::ALL {
            assert!(!dao.operations().is_empty(), "{} has no operations", dao.as_str());
        }
    }

    #[test]
    fn annotated_sql_helpers_map_to_query() {
        assert_eq!(DaoOperation::DeleteAll.annotation(), DaoAnnotation::Query);
        assert_eq!(DaoOperation::QueryByUid.annotation(), DaoAnnotation::Query);
        assert_eq!(DaoOperation::Insert.annotation(), DaoAnnotation::Insert);
        assert_eq!(DaoOperation::Update.annotation(), DaoAnnotation::Update);
    }

    #[test]
    fn steam_account_dao_has_no_insert() {
        // SteamAccountDao in the reference only declares delete/deleteAll/getAll.
        let ops = DaoKind::SteamAccountDao.operations();
        assert!(ops.contains(&DaoOperation::DeleteAll));
        assert!(!ops.contains(&DaoOperation::Insert));
    }
}
