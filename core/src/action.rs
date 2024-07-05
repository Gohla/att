#[derive(Default, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum ActionStyle {
  #[default]
  Primary,
  Secondary,
  Success,
  Danger
}

#[derive(Default, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum ActionLayout {
  #[default]
  Normal,
  TableRow,
  TableRowIcon,
}


#[derive(Default, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct ActionDef {
  pub text: &'static str,
  pub font_name: Option<&'static str>, // TODO: abstract over icon/font name
  pub layout: ActionLayout,
  pub style: ActionStyle,
}

impl ActionDef {
  #[inline]
  pub const fn new(text: &'static str, font_name: Option<&'static str>, layout: ActionLayout, style: ActionStyle) -> Self {
    Self { text, font_name, layout, style }
  }
  #[inline]
  pub const fn from_text(text: &'static str) -> Self {
    Self::new(text, None, ActionLayout::Normal, ActionStyle::Primary)
  }
  #[inline]
  pub const fn from_table_row_text(text: &'static str) -> Self {
    Self::new(text, None, ActionLayout::TableRow, ActionStyle::Primary)
  }
  #[inline]
  pub const fn from_table_row_icon(icon: &'static str, font_name: &'static str) -> Self {
    Self::new(icon, Some(font_name), ActionLayout::TableRowIcon, ActionStyle::Primary)
  }

  #[inline]
  pub const fn with_layout(mut self, layout: ActionLayout) -> Self {
    self.layout = layout;
    self
  }
  #[inline]
  pub const fn with_normal_layout(self) -> Self { self.with_layout(ActionLayout::Normal) }
  #[inline]
  pub const fn with_table_row_layout(self) -> Self { self.with_layout(ActionLayout::TableRow) }
  #[inline]
  pub const fn with_table_row_icon_layout(self) -> Self { self.with_layout(ActionLayout::TableRowIcon) }

  #[inline]
  pub const fn with_style(mut self, style: ActionStyle) -> Self {
    self.style = style;
    self
  }
  #[inline]
  pub const fn with_primary_style(self) -> Self { self.with_style(ActionStyle::Primary) }
  #[inline]
  pub const fn with_secondary_style(self) -> Self { self.with_style(ActionStyle::Secondary) }
  #[inline]
  pub const fn with_success_style(self) -> Self { self.with_style(ActionStyle::Success) }
  #[inline]
  pub const fn with_danger_style(self) -> Self { self.with_style(ActionStyle::Danger) }
}

pub trait Action {
  fn is_disabled(&self) -> bool;

  type Request;
  fn request(&self) -> Self::Request;
}

pub struct ActionWithDef<'a, A> {
  pub definition: &'a ActionDef,
  pub action: A,
}
impl<'a, A: Action> From<(&'a ActionDef, A)> for ActionWithDef<'a, A> {
  #[inline]
  fn from((definition, action): (&'a ActionDef, A)) -> Self {
    ActionWithDef { definition, action }
  }
}
