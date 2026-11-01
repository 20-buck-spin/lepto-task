use std::time::Duration;

use leptos::{logging, prelude::*, task::spawn_local};

use crate::server_funcs::task_runs::run_task;

#[component]
pub fn RunTaskBtn(task_id: i64, update_running_tasks_trigger: RwSignal<i32>) -> impl IntoView {
    let is_loading = RwSignal::new(false);
    view! {
        <button
            prop:disabled= move || is_loading.get()
            on:click = move |_| {
                            is_loading.set(true);
                            spawn_local(async move {
                                if let Err(err) = run_task(task_id).await {
                                    logging::error!("Failed to launch task. Err {}", err);
                                } else {
                                    update_running_tasks_trigger.update(|tr| *tr+=1);
                                };
                                set_timeout(
                                    move || is_loading.set(false),
                                    Duration::from_secs(1),
                                );
                            });
                        }
        >{
                {move || if is_loading.get(){"Running"} else {"Run"}}
        }</button>
    }
}
