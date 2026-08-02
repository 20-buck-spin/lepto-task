use chrono::Utc;
use leptos::{logging, prelude::*, task::spawn_local};

use crate::{
    frontend::{
        components::{
            dashboard_comps::run_task_btn::RunTaskBtn, empty_table::EmptyTable,
            table_skeleton::TableSkeleton,
        },
        loading_state::LoadingState,
    },
    models::queued_task::QueuedTask,
    server_funcs::tasks::get_queued_tasks,
    utils::format_duration,
    ws::{default_channel_names::QUEUED_TASKS_INFO_CHANNEL, WsClientContext, WsMessage},
};

#[component]
pub fn QueuedTasks(update_running_tasks_trigger: RwSignal<i32>) -> impl IntoView {
    let queued_tasks: RwSignal<Option<Vec<QueuedTask>>> = RwSignal::new(None);
    let ws = expect_context::<WsClientContext>();
    Effect::new(move |_| {
        untrack(move || {
            // fetch on load
            spawn_local(async move {
                match get_queued_tasks().await {
                    Ok(tasks) => queued_tasks.set(Some(tasks)),
                    Err(err) => logging::error!("Failed to fetch queued tasks. Err {}", err),
                }
            });

            // sub to ws
            ws.send.run(WsMessage::Subscribe {
                channel: QUEUED_TASKS_INFO_CHANNEL.into(),
            });
        });
    });

    on_cleanup(move || {
        ws.send.run(WsMessage::Unsubscribe {
            channel: QUEUED_TASKS_INFO_CHANNEL.into(),
        });
    });
    Effect::new(move |_| {
        if let Some(msg) = ws.latest_message.get() {
            match msg {
                WsMessage::Data { channel, payload } if &channel == QUEUED_TASKS_INFO_CHANNEL => {
                    match serde_json::from_value::<Vec<QueuedTask>>(payload.clone()) {
                        Ok(tasks) => *queued_tasks.write() = Some(tasks),
                        Err(e) => {
                            leptos::logging::error!(
                                "Failed to deserialize system info message. Message: {} err {}",
                                payload,
                                e
                            )
                        }
                    }
                }
                WsMessage::SubscribeFail { channel, reason }
                    if &channel == QUEUED_TASKS_INFO_CHANNEL =>
                {
                    leptos::logging::error!("Sub failed for {}: {}", channel, reason);
                }
                _ => {}
            }
        }
    });

    let loading_state = Signal::derive(move || {
        queued_tasks.with(|tasks_opt| match tasks_opt.as_ref() {
            None => LoadingState::Loading,
            Some(tasks) if tasks.is_empty() => LoadingState::Empty,
            Some(tasks) => LoadingState::Ready(tasks.clone()),
        })
    });

    view! {
            {move || {
                match loading_state.get() {
                    LoadingState::Loading => view!{
                        <TableSkeleton cols_len=3 rows_len=3/>
                    }.into_any(),
                    LoadingState::Empty => view!{
                        <EmptyTable display_message="No queued tasks found".to_string()/>
                    }.into_any(),
                    LoadingState::Ready(tasks) => view!{
                        <div class="table-outer-container">
                            <div class="table-inner-container">
                                    <table>
                                        <thead>
                                            <tr>
                                                <th>ID</th>
                                                <th>Name</th>
                                                <th>Command</th>
                                                <th>Working Dir</th>
                                                <th>Executes In</th>
                                                <th></th>
                                            </tr>
                                        </thead>

                                        <tbody>
                                            {
                                                let now = Utc::now();
                                                tasks.into_iter().map(
                                                    |task| {
                                                        view!{
                                                            <tr>
                                                                <td>{{task.task.id}}</td>
                                                                <td>{{task.task.name}}</td>
                                                                <td>{{task.task.command}}</td>
                                                                <td>{{task.task.working_dir}}</td>
                                                                <td>{{format_duration(task.next_exec_dt - now)}}</td>
                                                                <td>
                                                                    <RunTaskBtn task_id={task.task.id.clone()} update_running_tasks_trigger={update_running_tasks_trigger}/>
                                                                </td>
                                                            </tr>
                                                        }
                                                    }
                                                ).collect::<Vec<_>>()
                                            }
                                        </tbody>

                                    </table>
                            </div>
                    </div>}.into_any()
                }
            }}
    }
}
