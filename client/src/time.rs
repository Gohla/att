use std::time::Duration;

#[cfg(not(target_arch = "wasm32"))]
pub type Instant = std::time::Instant;
#[cfg(target_arch = "wasm32")]
pub type Instant = web_time::Instant;

#[cfg(not(target_arch = "wasm32"))]
pub async fn sleep(duration: Duration) {
  tokio::time::sleep(duration.into()).await;
}
#[cfg(target_arch = "wasm32")]
pub async fn sleep(duration: Duration) {
  gloo_timers::future::sleep(duration).await;
}
