use std::sync::Arc;

use axum::{
  extract::{
    ws::{Message, WebSocket},
    Extension, WebSocketUpgrade,
  },
  response::IntoResponse,
};
use futures::{SinkExt, StreamExt};

use crate::model::AppState;

fn get_username_if_not_exists(state: &Arc<AppState>, name: &String) -> Option<String> {
  let mut user_set = state.user_set.lock().unwrap();
  if user_set.contains(name) {
    return None;
  }
  user_set.insert(name.to_owned());
  Some(name.to_owned())
}

pub async fn websocket_handler(ws: WebSocketUpgrade, Extension(state): Extension<Arc<AppState>>) -> impl IntoResponse {
  ws.on_upgrade(|socket| handle_websocket(socket, state))
}

async fn handle_websocket(socket: WebSocket, state: Arc<AppState>) {
  let (mut sender, mut receiver) = socket.split();
  let mut username = String::new();

  while let Some(Ok(msg)) = receiver.next().await {
    if let Message::Text(name) = msg {
      if let Some(new_user) = get_username_if_not_exists(&state, &name) {
        username = new_user;
      }
      if username.is_empty() {
        let _ = sender.send(Message::Text("username already taken".to_string())).await;
        return; /* finish the function execution */
      } else {
        break; /* let's go with the user after the loop */
      }
    }
  }

  /* after the loop */
  let mut subscription = state.tx.subscribe();

  let msg = format!("{} joined", username);
  let _ = state.tx.send(msg);

  /* whenever, a broadcast is recieved, send to socket */
  let mut send_task = tokio::spawn(async move {
    while let Ok(incoming_msg) = subscription.recv().await {
      if sender.send(Message::Text(incoming_msg)).await.is_err() {
        break;
      }
    }
  });

  let tx = state.tx.clone();
  let name = username.clone();

  /* save incoming texts to AppState tx */
  let mut recv_task = tokio::spawn(async move {
    while let Some(Ok(Message::Text(text))) = receiver.next().await {
      let _ = tx.send(format!("{}: {}", name, text));
    }
  });

  /* when user leaves */
  tokio::select! {
    _=(&mut send_task) => recv_task.abort(),
    _=(&mut recv_task) => send_task.abort(),
  };
  let msg = format!("{} left.", username);
  let _ = state.tx.send(msg);
  state.user_set.lock().unwrap().remove(&username);
}
