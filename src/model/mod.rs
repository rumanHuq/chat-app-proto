use std::{collections::HashSet, sync::Mutex};

use tokio::sync::broadcast;
pub struct AppState {
  pub user_set: Mutex<HashSet<String>>,
  pub tx: broadcast::Sender<String>,
}
