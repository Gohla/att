use std::future::Future;

use crate::action::{Action, ActionDef, ActionWithDef};
use crate::query::{Query, QueryMessage};
use crate::util::maybe_send::MaybeSend;

/// Service that sends requests and processes responses.
///
/// Requests are [sent](Self::send), creating a future that returns a [`Response`](Self::Response) on completion.
/// Responses must be [processed](Self::process).
pub trait Service {
  type Request;
  type Response;

  /// Send `request`, possibly creating a future that produces a response when completed. The response must be
  /// [processed](Self::process).
  fn send(&mut self, request: Self::Request) -> Option<impl Future<Output=Self::Response> + MaybeSend + 'static>;

  /// Process `response` (that a future, created by [send](Self::send), returned on completion) into `self`. This
  /// possibly creates a future that must be processed again.
  fn process(&mut self, response: Self::Response) -> Option<impl Future<Output=Self::Response> + MaybeSend + 'static>;
}

#[macro_export]
macro_rules! forward_service_impl {
  ($src_ty:ty, $src:ident, $dst_ty:ty) => {
    impl $crate::service::Service for $dst_ty {
      type Request = <$src_ty as $crate::service::Service>::Request;
      type Response = <$src_ty as $crate::service::Service>::Response;

      #[inline]
      fn send(
        &mut self,
        request: Self::Request
      ) -> Option<impl std::future::Future<Output=Self::Response> + $crate::util::maybe_send::MaybeSend + 'static> {
        self.$src.send(request)
      }
      #[inline]
      fn process(
        &mut self,
        response: Self::Response
      ) -> Option<impl std::future::Future<Output=Self::Response> + $crate::util::maybe_send::MaybeSend + 'static> {
        self.$src.process(response)
      }
    }
  };
}


pub trait Catalog: Service {
  type Data;

  fn len(&self) -> usize;

  fn get(&self, index: usize) -> Option<&Self::Data>;

  fn iter(&self) -> impl Iterator<Item=&Self::Data>;


  type Query: Query;

  fn query(&self) -> &Self::Query;

  fn query_config(&self) -> &<Self::Query as Query>::Config;

  fn request_update(&self, message: QueryMessage) -> Self::Request;
}

#[macro_export]
macro_rules! forward_catalog_impl {
  ($src_ty:ty, $src:ident, $dst_ty:ty) => {
    impl $crate::service::Catalog for $dst_ty {
      type Data = <$src_ty as $crate::service::Catalog>::Data;

      #[inline]
      fn len(&self) -> usize { self.$src.len() }
      #[inline]
      fn get(&self, index: usize) -> Option<&Self::Data> { self.$src.get(index) }
      #[inline]
      fn iter(&self) -> impl Iterator<Item=&Self::Data> { self.$src.iter() }

      type Query = <$src_ty as $crate::service::QueryableCatalog>::Query;

      #[inline]
      fn query(&self) -> &Self::Query { self.$src.query() }
      #[inline]
      fn query_config(&self) -> &<Self::Query as $crate::query::Query>::Config { self.$src.query_config() }
      #[inline]
      fn request_update(&self, message: $crate::query::QueryMessage) -> Self::Request { self.$src.request_update(message) }
    }
  };
}


pub trait ServiceActions<S: Service> {
  fn action_definitions(&self, service: &S) -> &[ActionDef];

  fn actions(&self, service: &S) -> impl IntoIterator<Item=impl Action<Request=S::Request>>;

  #[inline]
  fn actions_with_definitions(&self, service: &S) -> impl Iterator<Item=ActionWithDef<impl Action<Request=S::Request>>> {
    self.action_definitions(service).iter().zip(self.actions(service)).map(Into::into)
  }
}

pub trait DataActions<S: Service + Catalog> {
  fn data_action_definitions(&self, service: &S) -> &[ActionDef];

  fn data_action<'d>(&self, service: &S, index: usize, data: &'d S::Data) -> Option<impl Action<Request=S::Request> + 'd>;

  fn data_action_with_definition<'d>(&self, service: &S, index: usize, data: &'d S::Data) -> Option<ActionWithDef<impl Action<Request=S::Request> + 'd>> {
    match (self.data_action_definitions(service).get(index), self.data_action(service, index, data)) {
      (Some(definition), Some(action)) => Some(ActionWithDef { definition, action }),
      _ => None
    }
  }
}
