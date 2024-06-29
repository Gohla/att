use hashlink::LinkedHashMap;

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
impl FacetType {
  pub fn default_value(&self) -> Option<FacetValue> {
    match self {
      FacetType::Boolean { default_value } => default_value.map(|default| FacetValue::Boolean(default)),
      FacetType::String { default_value, .. } => default_value.as_ref().map(|default| FacetValue::String(default.clone()))
    }
  }
}

#[derive(Debug)]
pub struct FacetDef {
  pub label: &'static str,
  pub facet_type: FacetType,
}
impl FacetDef {
  #[inline]
  pub fn new(header: &'static str, facet_type: FacetType) -> Self {
    Self { label: header, facet_type }
  }

  #[inline]
  fn create_facet(&self) -> Facet {
    Facet { value: self.facet_type.default_value() }
  }
}

#[derive(Default, Debug)]
pub struct QueryDef {
  facet_defs: LinkedHashMap<&'static str, FacetDef>,
}
impl QueryDef {
  #[inline]
  pub fn new() -> Self { Self::default() }

  #[inline]
  pub fn with_facet_def(mut self, facet_id: &'static str, facet_def: FacetDef) -> Self {
    self.facet_defs.insert(facet_id, facet_def);
    self
  }

  #[inline]
  pub fn add_facet_def(&mut self, facet_id: &'static str, facet_def: FacetDef) -> &mut Self {
    self.facet_defs.insert(facet_id, facet_def);
    self
  }


  #[inline]
  pub fn facet_defs_len(&self) -> usize {
    self.facet_defs.len()
  }

  #[inline]
  pub fn facet_def(&self, facet_id: &'static str) -> Option<&FacetDef> {
    self.facet_defs.get(facet_id)
  }

  #[inline]
  pub fn facet_defs(&self) -> impl Iterator<Item=(&'static str, &FacetDef)> {
    self.facet_defs.iter().map(|(facet_id, facet_def)| (*facet_id, facet_def))
  }


  #[inline]
  pub fn create_query(&self) -> Query {
    let mut facets = LinkedHashMap::with_capacity(self.facet_defs.len());
    for (facet_id, facet_def) in &self.facet_defs {
      facets.insert(*facet_id, facet_def.create_facet());
    }
    Query { facets }
  }
}


/// Value of a facet.
#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub enum FacetValue {
  Boolean(bool),
  String(String),
}

/// Facet of a query.
#[derive(Debug)]
pub struct Facet {
  value: Option<FacetValue>,
}

impl Facet {
  #[inline]
  pub fn new(value: Option<FacetValue>) -> Self {
    Self { value }
  }

  #[inline]
  pub fn new_none() -> Self {
    Self::new(None)
  }

  #[inline]
  pub fn new_boolean(boolean: bool) -> Self {
    Self::new(Some(FacetValue::Boolean(boolean)))
  }

  #[inline]
  pub fn new_string(string: String) -> Self {
    Self::new(Some(FacetValue::String(string)))
  }


  #[inline]
  pub fn as_bool(&self) -> Option<bool> {
    self.value.as_ref().map(|v| match v {
      FacetValue::Boolean(b) => *b,
      v => panic!("{:?} is not a boolean", v),
    })
  }

  #[inline]
  pub fn as_string(&self) -> Option<&String> {
    self.value.as_ref().map(|v| match v {
      FacetValue::String(s) => s,
      v => panic!("{:?} is not a string", v),
    })
  }

  #[inline]
  pub fn as_str(&self) -> Option<&str> {
    self.value.as_ref().map(|v| match v {
      FacetValue::String(s) => s.as_str(),
      v => panic!("{:?} is not a string", v),
    })
  }


  #[inline]
  pub fn value(&self) -> Option<&FacetValue> {
    self.value.as_ref()
  }

  #[inline]
  pub fn value_mut(&mut self) -> &mut Option<FacetValue> {
    &mut self.value
  }

  #[inline]
  pub fn set_value(&mut self, value: Option<FacetValue>) {
    self.value = value
  }


  #[inline]
  pub fn set_from(&mut self, facet: Facet) {
    self.value = facet.value
  }
}

/// A faceted search query.
#[derive(Debug)]
pub struct Query {
  facets: LinkedHashMap<&'static str, Facet>,
}

impl Query {
  #[inline]
  pub fn facets_len(&self) -> usize {
    self.facets.len()
  }


  #[inline]
  pub fn facet(&self, facet_id: &'static str) -> Option<&Facet> {
    self.facets.get(facet_id)
  }

  #[inline]
  pub fn facet_mut(&mut self, facet_id: &'static str) -> Option<&mut Facet> {
    self.facets.get_mut(facet_id)
  }


  #[inline]
  pub fn facets(&self) -> impl Iterator<Item=(&'static str, &Facet)> {
    self.facets.iter().map(|(facet_id, facet)| (*facet_id, facet))
  }

  #[inline]
  pub fn facets_mut(&mut self) -> impl Iterator<Item=(&'static str, &mut Facet)> {
    self.facets.iter_mut().map(|(facet_id, facet)| (*facet_id, facet))
  }

  #[inline]
  pub fn facets_with_defs<'a>(&'a self, query_def: &'a QueryDef) -> impl Iterator<Item=(&'static str, &'a FacetDef, &'a Facet)> + 'a {
    self.facets().flat_map(|(facet_id, facet)| query_def.facet_def(facet_id).map(|facet_def| (facet_id, facet_def, facet)))
  }

  #[inline]
  pub fn facets_with_defs_mut<'a>(&'a mut self, query_def: &'a QueryDef) -> impl Iterator<Item=(&'static str, &'a FacetDef, &'a mut Facet)> + 'a {
    self.facets_mut().flat_map(|(facet_id, facet)| query_def.facet_def(facet_id).map(|facet_def| (facet_id, facet_def, facet)))
  }
}
