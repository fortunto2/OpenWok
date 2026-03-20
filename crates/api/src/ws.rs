use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::{Path, State};
use axum::response::Response;
use futures_util::{SinkExt, StreamExt};

use crate::state::AppState;

pub async fn order_updates(
    ws: WebSocketUpgrade,
    Path(order_id): Path<String>,
    State(state): State<AppState>,
) -> Response {
    ws.on_upgrade(move |socket| handle_order_ws(socket, order_id, state))
}

async fn handle_order_ws(socket: WebSocket, order_id: String, state: AppState) {
    let (mut sender, mut receiver) = socket.split();
    let mut rx = state.order_events.subscribe();

    // Send matching order events to the client
    let oid = order_id.clone();
    let mut send_task = tokio::spawn(async move {
        while let Ok(event) = rx.recv().await {
            if event.order_id == oid {
                let json = serde_json::to_string(&event).unwrap_or_default();
                if sender.send(Message::Text(json.into())).await.is_err() {
                    break;
                }
            }
        }
    });

    // Keep connection alive — close when client disconnects
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(_msg)) = receiver.next().await {
            // Client pings or closes — just keep reading
        }
    });

    tokio::select! {
        _ = &mut send_task => recv_task.abort(),
        _ = &mut recv_task => send_task.abort(),
    }
}
