use leptos::{html::Table, prelude::*};
use std::collections::VecDeque;

use crate::{
    frontend::components::{
        empty_table::EmptyTable, task_run_logs_comps::raw_log_modal::RawLogModal,
    },
    models::task_models::LogEntry,
    utils::get_run_logs_channel_name,
    ws::{WsClientContext, WsMessage},
};

#[component]
pub fn TaskRunLiveLogs(run_id: i64) -> impl IntoView {
    let ws = expect_context::<WsClientContext>();
    let show_raw_log_modal: RwSignal<bool> = RwSignal::new(false);
    let raw_log_for_display: RwSignal<String> = RwSignal::new(String::new());
    let live_logs: RwSignal<VecDeque<(usize, LogEntry)>> =
        RwSignal::new(VecDeque::with_capacity(200));

    let element = NodeRef::<Table>::new();
    let auto_scroll = RwSignal::new(true);
    let is_programmatic_scroll = RwSignal::new(false);

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

    // 1. Detect user scroll vs programmatic scroll
    let on_table_scroll = move |_| {
        // Ignore scroll events triggered by our own auto-scroller
        if is_programmatic_scroll.get_untracked() {
            return;
        }

        if let Some(el) = element.get_untracked() {
            let scroll_top = el.scroll_top();
            let scroll_height = el.scroll_height();
            let client_height = el.client_height();

            // Allow a small threshold (e.g. 10px) for sub-pixel rendering differences
            let is_at_bottom = (scroll_height - scroll_top - client_height).abs() <= 10;
            auto_scroll.set(is_at_bottom);
        }
    };

    // 2. Perform auto-scroll when `live_logs` updates
    Effect::new(move |_| {
        // Track changes to live_logs
        let logs_not_empty = live_logs.with(|logs| !logs.is_empty());

        if logs_not_empty && auto_scroll.get_untracked() {
            request_animation_frame(move || {
                if let Some(el) = element.get_untracked() {
                    is_programmatic_scroll.set(true);
                    el.set_scroll_top(el.scroll_height());

                    // Reset the guard on the next frame after the scroll event has processed
                    request_animation_frame(move || {
                        is_programmatic_scroll.set(false);
                    });
                }
            });
        }
    });

    // Add to rows
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
                                        live_logs.pop_front();
                                    };
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
                    node_ref=element
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
