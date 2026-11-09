use std::collections::VecDeque;

use leptos::{html::Table, prelude::*};
use leptos_use::{use_scroll, UseScrollReturn};

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

    let UseScrollReturn {
        arrived_state,
        is_scrolling,
        ..
    } = use_scroll(element);

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

    Effect::new(move |_| {
        let arrived = arrived_state.get();
        if element.get().is_some() && is_scrolling.get() {
            auto_scroll.set(arrived.bottom);
        }
    });

    Effect::new(move |_| {
        let logs_not_empty = live_logs.with(|live_logs| !live_logs.is_empty());
        let maybe_el = element.get();
        if logs_not_empty && auto_scroll.get_untracked() {
            if let Some(el) = maybe_el {
                request_animation_frame(move || {
                    el.set_scroll_top(el.scroll_height());
                });
            }
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
            <div class="live-logs-table-container" >
                <table class="live-logs-table" node_ref=element>
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
                                        <td>{view!{
                                            <button on:click= move |_| {show_raw_log_modal.set(true); raw_log_for_display.set(log.raw_payload.clone())}
                                            >View Raw Logs</button>
                                        }}</td>
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
