use leptos::{ev::WheelEvent, html::Table, prelude::*};
use std::collections::VecDeque;

use crate::{
    frontend::components::{
        empty_table::EmptyTable, task_run_logs_comps::raw_log_modal::RawLogModal,
    },
    models::task_models::LogEntry,
    utils::get_run_logs_channel_name,
    ws::{WsClientContext, WsMessage},
};

const MAX_LOGS_AT_BOTTOM: usize = 200;
const MAX_LOGS_WHILE_PAUSED: usize = 2000;

#[component]
pub fn TaskRunLiveLogs(run_id: i64) -> impl IntoView {
    let ws = expect_context::<WsClientContext>();
    let show_raw_log_modal: RwSignal<bool> = RwSignal::new(false);
    let raw_log_for_display: RwSignal<String> = RwSignal::new(String::new());
    let live_logs: RwSignal<VecDeque<(usize, LogEntry)>> =
        RwSignal::new(VecDeque::with_capacity(MAX_LOGS_AT_BOTTOM));

    let element = NodeRef::<Table>::new();
    let auto_scroll = RwSignal::new(true);

    let channel_name = get_run_logs_channel_name(&run_id);

    let channel_name_clone = channel_name.clone();
    Effect::new(move |_| {
        let channel = channel_name_clone.clone();
        untrack(move || {
            ws.send.run(WsMessage::Subscribe { channel });
        });
    });

    let channel_name_clone = channel_name.clone();
    on_cleanup(move || {
        ws.send.run(WsMessage::Unsubscribe {
            channel: channel_name_clone,
        });
    });

    // 1. Mouse wheel up immediately disengages auto-scroll
    let on_table_wheel = move |ev: WheelEvent| {
        if ev.delta_y() < 0.0 {
            auto_scroll.set(false);
        }
    };

    // 2. Scroll event checks if the user is at (or near) the bottom
    let on_table_scroll = move |_| {
        if let Some(el) = element.get_untracked() {
            let scroll_top = el.scroll_top();
            let scroll_height = el.scroll_height();
            let client_height = el.client_height();

            let distance_from_bottom = scroll_height - scroll_top - client_height;

            if distance_from_bottom <= 30 {
                if !auto_scroll.get_untracked() {
                    auto_scroll.set(true);
                }
            } else if distance_from_bottom > 80 {
                // User dragged the scrollbar thumb up
                if auto_scroll.get_untracked() {
                    auto_scroll.set(false);
                }
            }
        }
    };

    // 3. Auto-scroll synchronously after DOM updates
    Effect::new(move |_| {
        // Subscribe to both live_logs and auto_scroll
        let logs_len = live_logs.with(|logs| logs.len());
        let should_scroll = auto_scroll.get();

        if logs_len > 0 && should_scroll {
            // Trim excess logs accumulated while paused
            if logs_len > MAX_LOGS_AT_BOTTOM {
                live_logs.update(|logs| {
                    while logs.len() > MAX_LOGS_AT_BOTTOM {
                        logs.pop_front();
                    }
                });
            }

            if let Some(el) = element.get() {
                el.set_scroll_top(el.scroll_height());
            }
            request_animation_frame(move || {
                if auto_scroll.get_untracked() {
                    if let Some(el) = element.get_untracked() {
                        el.set_scroll_top(el.scroll_height());
                    }
                }
            });
        }
    });

    // 4. Add incoming WS logs
    Effect::new({
        let mut local_id_counter: usize = 0;
        move |_| {
            if let Some(msg) = ws.latest_message.get() {
                match msg {
                    WsMessage::Data { channel, payload } if &channel == &channel_name => {
                        match serde_json::from_value::<LogEntry>(payload.clone()) {
                            Ok(log_line) => {
                                let is_following = auto_scroll.get_untracked();
                                live_logs.update(|live_logs| {
                                    let limit = if is_following {
                                        MAX_LOGS_AT_BOTTOM
                                    } else {
                                        MAX_LOGS_WHILE_PAUSED
                                    };

                                    while live_logs.len() >= limit {
                                        live_logs.pop_front();
                                    }
                                    live_logs.push_back((local_id_counter, log_line));
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
        <Show when=move || show_raw_log_modal.get()>
            <RawLogModal on_close = move || {show_raw_log_modal.set(false)} raw_log=raw_log_for_display />
        </Show>

        <div class="table-header">
            <h3>"Live logs"</h3>
        </div>
        <Show
            when=move || live_logs.with(|l| !l.is_empty())
            fallback=|| view! { <EmptyTable display_message={"No logs captured".to_string()}/> }
        >
            <div class="live-logs-table-container">
                <table
                    class="live-logs-table"
                    style="overflow-anchor: none;"
                    node_ref=element
                    on:wheel=on_table_wheel
                    on:scroll=on_table_scroll
                >
                    <thead>
                        <tr>
                            <th>"Created"</th>
                            <th>"Level"</th>
                            <th>"Message"</th>
                            <th>""</th>
                        </tr>
                    </thead>
                    <tbody>
                        <For
                            each=move || live_logs.get()
                            key=|(id, _log)| *id
                            children= move |(_, log)| {
                                view! {
                                    <tr>
                                        <td>{log.timestamp.to_string()}</td>
                                        <td>{log.level.to_string()}</td>
                                        <td>{log.message.clone()}</td>
                                        <td>
                                            <button on:click= move |_| {
                                                show_raw_log_modal.set(true);
                                                raw_log_for_display.set(log.raw_payload.clone());
                                            }>
                                                "View Raw Logs"
                                            </button>
                                        </td>
                                    </tr>
                                }
                            }
                        />
                    </tbody>
                </table>
            </div>
        </Show>
    }
}
