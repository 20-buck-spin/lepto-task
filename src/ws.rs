use leptos::{
    callback::Callback,
    logging,
    prelude::*,
    reactive::{owner::expect_context, signal::RwSignal},
    server_fn::{codec::JsonEncoding, BoxedStream, ServerFnError, Websocket},
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub mod default_channel_names;

#[cfg(feature = "ssr")]
pub mod ws_context;

#[derive(Serialize, Deserialize, Clone)]
pub enum WsMessage {
    Subscribe { channel: String },
    Unsubscribe { channel: String },
    SubscribeSuccess { channel: String },
    SubscribeFail { channel: String, reason: String },
    Data { channel: String, payload: Value },
}

#[derive(Clone)]
pub struct WsClientContext {
    pub latest_message: RwSignal<Option<WsMessage>>,
    pub send: Callback<WsMessage>,
}

#[server(protocol=Websocket<JsonEncoding, JsonEncoding>)]
pub async fn pubsub_websocket(
    input: BoxedStream<WsMessage, ServerFnError>,
) -> Result<BoxedStream<WsMessage, ServerFnError>, ServerFnError> {
    use futures::{channel::mpsc as fmpsc, SinkExt, StreamExt};
    use tokio::sync::mpsc as tmpsc;
    use uuid::Uuid;
    use ws_context::WsContext;

    let mut input = input;

    let ws_context = expect_context::<WsContext>();
    let client_id = Uuid::new_v4();

    let (cl_tx, cl_rx) = fmpsc::channel::<Result<WsMessage, ServerFnError>>(100);
    let (in_tx, mut in_rx) = tmpsc::channel::<WsMessage>(100);

    let mut cl_tx_clone = cl_tx.clone();
    let mut send_task = tokio::spawn(async move {
        while let Some(msg) = in_rx.recv().await {
            if cl_tx_clone.send(Ok(msg)).await.is_err() {
                break;
            }
        }
    });

    let ws_context_clone = ws_context.clone();
    let mut client_tx_clone_2 = cl_tx.clone();
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = input.next().await {
            match msg {
                WsMessage::Subscribe { channel } => {
                    if let Some(subs) = ws_context_clone.channels.get(&channel) {
                        subs.insert(client_id, in_tx.clone());
                        if let Err(err) = client_tx_clone_2
                            .send(Ok(WsMessage::SubscribeSuccess { channel }))
                            .await
                        {
                            // Add logging here
                            logging::error!(
                                "Failed to send subscribe message to client. Err {}",
                                err
                            )
                        };
                    } else {
                        if let Err(err) = client_tx_clone_2
                            .send(Ok(WsMessage::SubscribeFail {
                                channel: channel.clone(),
                                reason: format!("Channel {} not found", channel),
                            }))
                            .await
                        {
                            // Add logging here
                            logging::error!(
                                "Failed to send subscribe failed message to client. Err {}",
                                err
                            )
                        };
                    }
                }
                WsMessage::Unsubscribe { channel } => {
                    if let Some(subs) = ws_context_clone.channels.get(&channel) {
                        subs.remove(&client_id);
                    }
                }
                _ => {}
            }
        }
    });

    tokio::spawn(async move {
        tokio::select!(
            _ = (&mut send_task) => recv_task.abort(),
            _ = (&mut recv_task) => send_task.abort()
        );

        for channel_entry in ws_context.channels.iter_mut() {
            channel_entry.remove(&client_id);
        }
    });

    Ok(cl_rx.into())
}
