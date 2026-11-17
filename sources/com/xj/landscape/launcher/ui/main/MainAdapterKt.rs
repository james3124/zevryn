//! Portable part of `com.xj.landscape.launcher.ui.main.MainAdapterKt`.
//!
//! The decompiled adapter binds `CardLineData` rows to Android view holders.
//! The view classes themselves are not portable, but the *dispatch rules* are:
//! which holder a card type maps to, the fixed-image view-type key, the
//! empty-row filter, and the "same vertical type" merge predicate used by
//! `MyContainerView.setModels()`. Those rules are modeled here.

use super::me::{CardLine, CardLineType};

/// Holder families selected by `MainAdapterKt.g()` / `MainAdapterKt.i()`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewHolderKind {
    Platform,
    CloudPlayPlatform,
    FixedImage,
    BannerImage,
    Album,
    AlbumGames,
    PcEmulatorButtons,
    PcEmulatorInfo,
}

/// `MainAdapterKt.g(z3, function1, data)` restricted to `CardLineData` rows.
/// Returns `None` where the Java code would fall back to the caller supplied
/// `function1` (or `null`) for unknown types.
pub fn view_holder_for(card_type: Option<CardLineType>, is_cloud_play: bool) -> Option<ViewHolderKind> {
    use CardLineType::*;
    match card_type? {
        Platform => Some(if is_cloud_play {
            ViewHolderKind::CloudPlayPlatform
        } else {
            ViewHolderKind::Platform
        }),
        FixedImage => Some(ViewHolderKind::FixedImage),
        BannerImage => Some(ViewHolderKind::BannerImage),
        Album => Some(ViewHolderKind::Album),
        AlbumGames => Some(ViewHolderKind::AlbumGames),
        Unknown(_) => None,
    }
}

const FIXED_IMAGE_HOLDER_CLASS: &str =
    "com.xj.landscape.launcher.ui.main.viewholders.FixedImageViewHolder";

/// The string view-type key returned for a fixed-image row by
/// `MainAdapterKt.g()`: `ClassName_<fixed_card_size>_<aspect_ratio>` with
/// defaults `3` and `0.56f`. Non-positive sizes fall back to `3`.
pub fn fixed_image_view_type_key(line: &CardLine) -> String {
    let size = line.fixed_card_size.filter(|n| *n > 0).unwrap_or(3);
    let ratio = line.aspect_ratio.unwrap_or(0.56) as f32;
    format!("{}_{}_{}", FIXED_IMAGE_HOLDER_CLASS, size, ratio)
}

/// `MainAdapterKt.j(int, Object)`: keep every row except empty FIXED_IMAGE
/// rows (`card_type == 3` and an empty `card_list`).
pub fn should_show_line(line: &CardLine) -> bool {
    let is_fixed_image = line.card_type.map(|t| t.code()) == Some(CardLineType::FixedImage.code());
    !is_fixed_image || !line.cards.is_empty()
}

/// `CardLineData.isSameVerticalType(obj)`, used by `MyContainerView.setModels`
/// to merge consecutive vertical fixed-image rows.
pub fn is_same_vertical_type(a: &CardLine, b: &CardLine) -> bool {
    if a.id != b.id || a.card_type != b.card_type {
        return false;
    }
    if !a.is_vertical || !b.is_vertical {
        return false;
    }
    if a.fixed_card_size != b.fixed_card_size {
        return false;
    }
    // The Java additionally requires card_type == FIXED_IMAGE && isVertical == 1
    // and a positive parsed fixed_card_size.
    a.card_type == Some(CardLineType::FixedImage)
        && a.fixed_card_size.map(|n| n > 0).unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use CardLineType::*;

    fn line(t: Option<CardLineType>, size: Option<i32>, ratio: Option<f64>, cards: usize) -> CardLine {
        CardLine {
            id: "r".into(),
            title: String::new(),
            desc: String::new(),
            card_type: t,
            bg_img: String::new(),
            aspect_ratio: ratio,
            fixed_card_size: size,
            is_vertical: true,
            is_text_outside: false,
            is_play_video: false,
            is_more: false,
            page: 0,
            page_size: 0,
            total: 0,
            cards: (0..cards)
                .map(|i| super::super::me::GameCard {
                    id: i.to_string(),
                    game_id: String::new(),
                    title: String::new(),
                    subtitle: String::new(),
                    content_img: String::new(),
                    game_cover_image: String::new(),
                    game_back_image: String::new(),
                    start_type: String::new(),
                    start_name: String::new(),
                    start_icon: String::new(),
                    steam_app_id: String::new(),
                    card_tag: String::new(),
                    topic_name: String::new(),
                    top_platform_type: String::new(),
                })
                .collect(),
        }
    }

    #[test]
    fn maps_card_types_to_holders() {
        assert_eq!(view_holder_for(Some(Platform), false), Some(ViewHolderKind::Platform));
        assert_eq!(view_holder_for(Some(Platform), true), Some(ViewHolderKind::CloudPlayPlatform));
        assert_eq!(view_holder_for(Some(FixedImage), false), Some(ViewHolderKind::FixedImage));
        assert_eq!(view_holder_for(Some(BannerImage), false), Some(ViewHolderKind::BannerImage));
        assert_eq!(view_holder_for(Some(Album), false), Some(ViewHolderKind::Album));
        assert_eq!(view_holder_for(Some(AlbumGames), false), Some(ViewHolderKind::AlbumGames));
        assert_eq!(view_holder_for(Some(Unknown(9)), false), None);
        assert_eq!(view_holder_for(None, false), None);
    }

    #[test]
    fn fixed_image_view_type_key_defaults() {
        let key = fixed_image_view_type_key(&line(Some(FixedImage), None, None, 0));
        assert!(key.ends_with("_3_0.56"), "unexpected key: {key}");
        let key = fixed_image_view_type_key(&line(Some(FixedImage), Some(0), Some(1.5), 0));
        assert!(key.ends_with("_3_1.5"), "unexpected key: {key}");
        let key = fixed_image_view_type_key(&line(Some(FixedImage), Some(4), Some(1.5), 0));
        assert!(key.ends_with("_4_1.5"), "unexpected key: {key}");
    }

    #[test]
    fn filters_empty_fixed_image_rows() {
        assert!(!should_show_line(&line(Some(FixedImage), None, None, 0)));
        assert!(should_show_line(&line(Some(FixedImage), None, None, 1)));
        assert!(should_show_line(&line(Some(Platform), None, None, 0)));
    }

    #[test]
    fn merges_same_vertical_fixed_image_rows() {
        let a = line(Some(FixedImage), Some(3), None, 1);
        let b = a.clone();
        assert!(is_same_vertical_type(&a, &b));
        let different_size = line(Some(FixedImage), Some(4), None, 1);
        assert!(!is_same_vertical_type(&a, &different_size));
        let platform = line(Some(Platform), Some(3), None, 1);
        assert!(!is_same_vertical_type(&a, &platform));
    }
}
