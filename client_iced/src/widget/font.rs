use iced::Font;

/// Fira Sans regular font bytes.
pub const FIRA_SANS_FONT_BYTES: &[u8] = include_bytes!("../../font/FiraSans-Regular.ttf");
/// Fira Sans regular font.
pub const FIRA_SANS_FONT: Font = Font::with_name("FiraSans-Regular");

/// [Bootstrap icon](https://icons.getbootstrap.com/) font bytes.
pub const ICON_FONT_BYTES: &[u8] = include_bytes!("../../font/bootstrap-icons.ttf");
/// [Bootstrap icon](https://icons.getbootstrap.com/) font.
pub const ICON_FONT: Font = Font::with_name("bootstrap-icons");
