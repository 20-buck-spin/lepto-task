use chrono::Utc;
use leptos::{logging, prelude::*, task::spawn_local};
use leptos_router::components::A;

use super::super::runs_comps::cancel_run_btn::CancelRunButton;
use crate::{
    frontend::{
        components::{empty_table::EmptyTable, table_skeleton::TableSkeleton},
        loading_state::LoadingState,
    },
    models::task_models::RunningTaskPublic,
    server_funcs::task_runs::get_running_tasks,
    utils::format_duration,
    ws::{default_channel_names::RUNNING_PROCESSES, WsClientContext, WsMessage},
};

#[component]
pub fn RunningTasks(update_running_tasks_trigger: RwSignal<i32>) -> impl IntoView {
    let running_tasks: RwSignal<Option<Vec<RunningTaskPublic>>> = RwSignal::new(None);
    let ws = expect_context::<WsClientContext>();

    let get_running_tasks_local = move || {
        spawn_local(async move {
            match get_running_tasks().await {
                Ok(running_tasks_list) => running_tasks.set(Some(running_tasks_list)),
                Err(err) => logging::log!("Failed to get running tasks. Err {}", err),
            }
        });
    };

    Effect::new(move |_| {
        get_running_tasks_local();
        untrack(move || {
            ws.send.run(WsMessage::Subscribe {
                channel: RUNNING_PROCESSES.into(),
            });
        });
    });

    Effect::new(move |_| {
        let _ = update_running_tasks_trigger.get();
        get_running_tasks_local();
    });

    on_cleanup(move || {
        ws.send.run(WsMessage::Unsubscribe {
            channel: RUNNING_PROCESSES.into(),
        });
    });
    Effect::new(move |_| {
        if let Some(msg) = ws.latest_message.get() {
            match msg {
                WsMessage::Data { channel, payload } if &channel == RUNNING_PROCESSES => {
                    match serde_json::from_value::<Vec<RunningTaskPublic>>(payload.clone()) {
                        Ok(tasks) => *running_tasks.write() = Some(tasks),
                        Err(e) => {
                            leptos::logging::error!(
                                "Failed to deserialize system info message. Message: {} err {}",
                                payload,
                                e
                            )
                        }
                    }
                }
                WsMessage::SubscribeFail { channel, reason } if &channel == RUNNING_PROCESSES => {
                    leptos::logging::error!("Sub failed for {}: {}", channel, reason);
                }
                _ => {}
            }
        }
    });

    let on_cancel = Callback::new(move |()| {
        running_tasks.set(None);
        spawn_local(async move {
            match get_running_tasks().await {
                Ok(running_tasks_list) => running_tasks.set(Some(running_tasks_list)),
                Err(err) => logging::log!("Failed to get running tasks. Err {}", err),
            }
        });
    });

    let loading_state = Signal::derive(move || {
        running_tasks.with(|tasks_opt| match tasks_opt.as_ref() {
            None => LoadingState::Loading,
            Some(tasks) if tasks.is_empty() => LoadingState::Empty,
            Some(tasks) => LoadingState::Ready(tasks.clone()),
        })
    });

    view! {
        {move || {
                match loading_state.get() {
                    LoadingState::Loading => {
                        view!{
                            <TableSkeleton cols_len=3 rows_len=3 />
                        }.into_any()
                    },
                    LoadingState::Empty => {
                        view!{
                            <EmptyTable display_message="No running tasks currently".to_string()/>
                        }.into_any()
                        },
                    LoadingState::Ready(tasks) => {
                        view!{

                            <div class="table-outer-container">
                                <div class="table-inner-container">

                                <table>
                                    <thead>
                                        <th>Run ID</th>
                                        <th>Task Name</th>
                                        <th>Duration</th>
                                        <th>Actions</th>
                                    </thead>

                                    <tbody>
                                    {
                                        let now = Utc::now();
                                        tasks.into_iter().map(|task| {
                                            view!{
                                                <tr>
                                                    <td>{{task.run_id.clone()}}</td>
                                                    <td>{{task.task.name}}</td>
                                                    <td>{{format_duration(now - task.started_at)}}</td>
                                                    <td class="btns-container">
                                                        <A href=format!("/task_runs/{}", task.run_id)>
                                                            <button>"View"</button>
                                                        </A>
                                                        <CancelRunButton run_id=task.run_id on_cancel=on_cancel/>
                                                    </td>
                                                </tr>
                                            }
                                        }).collect::<Vec<_>>()
                                    }
                                    </tbody>
                                </table>

                                </div>
                            </div>

                        }.into_any()
                    }
                }
            }
        }
    }
}
