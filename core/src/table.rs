use std::borrow::Cow;

/// Horizontal and vertical alignment.
#[derive(Default, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum Alignment {
  /// Align at the start of the axis.
  #[default]
  Start,
  /// Align at the center of the axis.
  Center,
  /// Align at the end of the axis.
  End,
}

/// Table column definition.
#[derive(Default, Clone, Copy, PartialEq, PartialOrd, Debug)]
pub struct ColumnDef {
  pub header: &'static str,
  pub width_fill_portion: f32,
  pub horizontal_alignment: Alignment,
  pub vertical_alignment: Alignment,
}

impl ColumnDef {
  #[inline]
  pub const fn new(header: &'static str, width_fill_portion: f32, horizontal_alignment: Alignment, vertical_alignment: Alignment) -> Self {
    Self { header, width_fill_portion, horizontal_alignment, vertical_alignment }
  }

  #[inline]
  pub const fn with_default_alignment(header: &'static str, width_fill_portion: f32) -> Self {
    Self { header, width_fill_portion, horizontal_alignment: Alignment::Start, vertical_alignment: Alignment::Start }
  }
}

/// Turn a value into a table row.
pub trait AsTableRow {
  const COLUMNS: &'static [ColumnDef];

  fn cell(&self, column_index: u8) -> Option<Cow<str>>;
}
