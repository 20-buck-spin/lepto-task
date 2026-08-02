use std::collections::VecDeque;

use leptos::prelude::*;

use crate::{
    frontend::components::empty_table::EmptyTable,
    models::task_models::LogEntry,
    utils::get_run_logs_channel_name,
    ws::{WsClientContext, WsMessage},
};

#[component]
pub fn TaskRunLiveLogs(run_id: i64) -> impl IntoView {
    let ws = expect_context::<WsClientContext>();

    let live_logs: RwSignal<VecDeque<(usize, LogEntry)>> =
        RwSignal::new(VecDeque::with_capacity(200));
    let channel_name = get_run_logs_channel_name(&run_id);

    let channel_name_clone = channel_name.clone();
    Effect::new(move |_| {
        let channel = channel_name_clone.clone();
        untrack(move || {
            ws.send.run(WsMessage::Subscribe { channel: channel });
        });
    });

    let channel_name_clone = channel_name.clone();
    on_cleanup(move || {
        ws.send.run(WsMessage::Unsubscribe {
            channel: channel_name_clone,
        });
    });

    Effect::new({
        let mut local_id_counter: usize = 0;
        move |_| {
            if let Some(msg) = ws.latest_message.get() {
                match msg {
                    WsMessage::Data { channel, payload } if &channel == &channel_name => {
                        match serde_json::from_value::<LogEntry>(payload.clone()) {
                            Ok(log_line) => {
                                live_logs.update(|live_logs| {
                                    if live_logs.len() == 200 {
                                        live_logs.pop_back();
                                    };
                                    live_logs.push_front((local_id_counter, log_line));
                                    local_id_counter += 1;
                                });
                            }
                            Err(e) => {
                                leptos::logging::error!(
                                    "Failed to deserialize system info message. Message: {} err {}",
                                    payload,
                                    e
                                )
                            }
                        }
                    }
                    WsMessage::SubscribeFail { channel, reason } if &channel == &channel_name => {
                        leptos::logging::error!("Sub failed for {}: {}", channel, reason);
                    }
                    _ => {}
                }
            }
        }
    });
    view! {
        <div class="table-header">
            <h3>"Live logs"</h3>
        </div>
        <Show
            when=move || !live_logs.with(|l| l.is_empty())
            fallback=|| view! { <EmptyTable display_message={"No logs captured".to_string()}/> }
        >
            <table class="live-logs-table">
                <thead>
                    <tr>
                        <th>"Created"</th>
                        <th>"Level"</th>
                        <th>"Message"</th>
                        <th>"Raw Payload"</th>
                    </tr>
                </thead>
                <tbody>
                    <For
                        each=move || live_logs.get()
                        key=|(id, _log)| *id
                        children=|(_, log)| {
                            view! {
                                <tr>
                                    <td>{log.timestamp.to_string()}</td>
                                    <td>{log.level.to_string()}</td>
                                    <td>{log.message.clone()}</td>
                                    <td>{log.raw_payload.to_string()}</td>
                                </tr>
                            }
                        }
                    />
                </tbody>
            </table>
        </Show>
    }
}
