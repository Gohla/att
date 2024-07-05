/// Query abstraction.
pub trait Query {
  const FACET_DEFS: &'static [FacetDef];

  type Config;
  fn should_show(config: &Self::Config, index: u8) -> bool;

  fn is_empty(&self, config: &Self::Config) -> bool;
  fn facet(&self, config: &Self::Config, index: u8) -> Option<FacetRef>;
  fn set_facet(&mut self, config: &Self::Config, index: u8, facet: Option<Facet>);
}


/// Query facet definition.
#[derive(Debug)]
pub struct FacetDef {
  pub label: &'static str,
  pub facet_type: FacetType,
}
impl FacetDef {
  #[inline]
  pub const fn new(header: &'static str, facet_type: FacetType) -> Self {
    Self { label: header, facet_type }
  }
}

/// Type of query facet.
#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub enum FacetType {
  Boolean {
    default_value: Option<bool>,
  },
  String {
    default_value: Option<String>,
    placeholder: Option<&'static str>,
  },
}


/// Query facet reference.
#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub enum FacetRef<'a> {
  Boolean(bool),
  String(&'a str),
}
impl<'a> FacetRef<'a> {
  #[inline]
  pub fn into_bool(self) -> Result<bool, Self> {
    let Self::Boolean(boolean) = self else {
      return Err(self);
    };
    Ok(boolean)
  }

  #[inline]
  pub fn into_str(self) -> Result<&'a str, Self> {
    let Self::String(str) = self else {
      return Err(self);
    };
    Ok(str)
  }
}

/// Query facet value.
#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub enum Facet {
  Boolean(bool),
  String(String),
}
impl Facet {
  #[inline]
  pub fn into_bool(self) -> Result<bool, Self> {
    let Self::Boolean(b) = self else {
      return Err(self);
    };
    Ok(b)
  }

  #[inline]
  pub fn into_string(self) -> Result<String, Self> {
    let Self::String(s) = self else {
      return Err(self);
    };
    Ok(s)
  }
}


/// Query message
#[derive(Debug)]
pub enum QueryMessage {
  /// Facet at `index` has been changed into `new_facet`.
  FacetChange {
    index: u8,
    new_facet: Option<Facet>,
  }
}
impl QueryMessage {
  #[inline]
  pub fn facet_change(facet_index: u8, new_facet: Option<Facet>) -> Self {
    Self::FacetChange { index: facet_index, new_facet }
  }
  #[inline]
  pub fn facet_change_bool(facet_index: u8, boolean: bool) -> Self {
    Self::facet_change(facet_index, Some(Facet::Boolean(boolean)))
  }
  #[inline]
  pub fn facet_change_string(facet_index: u8, string: String) -> Self {
    Self::facet_change(facet_index, Some(Facet::String(string)))
  }

  #[inline]
  pub fn update_query<Q: Query>(self, query: &mut Q, config: &Q::Config) {
    match self {
      QueryMessage::FacetChange { index, new_facet } => {
        query.set_facet(config, index, new_facet);
      }
    }
  }
}
