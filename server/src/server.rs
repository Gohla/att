use std::error::Error;
use std::future::Future;
use std::net::SocketAddr;

use axum::Router;
use axum_login::AuthManagerLayerBuilder;
use tower_http::trace::TraceLayer;
use tower_sessions::{Expiry, MemoryStore, SessionManagerLayer};
use tower_sessions::cookie::time::Duration;

use crate::crates::{self, Crates};
use crate::users::{self, Users};

#[derive(Clone)]
pub struct Server {
  users: Users,
  crates: Crates,
}

impl Server {
  pub fn new(users: Users, crates: Crates) -> Self {
    Self { users, crates }
  }

  pub async fn run(self, shutdown_signal: impl Future<Output=()> + Send + 'static) -> Result<(), Box<dyn Error>> {
    self.users.ensure_default_user_exists().await?;

    let session_store = MemoryStore::default();
    let session_layer = SessionManagerLayer::new(session_store)
      .with_expiry(Expiry::OnInactivity(Duration::days(30)))
      ;

    let authentication_layer = AuthManagerLayerBuilder::new(self.users.clone(), session_layer.clone())
      .build();

    let users_routes = users::router().with_state(());
    let crates_routes = crates::route::router()
      .with_state(self.crates);

    let api_routes = Router::new()
      .nest("/users", users_routes)
      .nest("/crates", crates_routes)
      ;

    let router = Router::new()
      .nest("/api", api_routes)
      .layer(session_layer)
      .layer(authentication_layer)
      .layer(TraceLayer::new_for_http())
      ;

    let addr = SocketAddr::from(([127, 0, 0, 1], 1337));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, router)
      .with_graceful_shutdown(shutdown_signal)
      .await?;

    Ok(())
  }
}
