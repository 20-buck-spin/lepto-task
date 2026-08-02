use crate::models::task_models::Task;
use crate::{frontend::components::modal::Modal, server_funcs::tasks::CreateUpdateTask};
use leptos::prelude::*;

#[component]
pub fn CreateUpdateTaskModal(
    on_close: impl FnMut() + 'static,
    mut on_success: impl FnMut() + 'static,
    selected_task: RwSignal<Option<Task>>,
) -> impl IntoView {
    let modal_title = move || match selected_task.get() {
        Some(task) => format!("Update Task {}", task.id),
        None => "Create Task".into(),
    };

    let submit_action = ServerAction::<CreateUpdateTask>::new();
    let pending = submit_action.pending();

    Effect::new(move || {
        if submit_action.value().get().is_some() {
            on_success()
        }
    });

    view! {
        <Modal modal_title=modal_title on_close=on_close>
            <div class="form-container">

                <ActionForm
                    attr:class="form"
                    action=submit_action>

                    <input type="numver" name="data[id]" hidden
                    value={selected_task.with_untracked(|sel_task| { sel_task.clone().map(|task| task.id)})}
                    />

                    <input type="text" name="data[name]" placeholder="Task name"
                        value={selected_task.with_untracked(|sel_task| { sel_task.clone().map(|task| task.name)})}
                    />

                    <input type="text" name="data[command]" placeholder="Command" required
                        value={selected_task.with_untracked(|sel_task| { sel_task.clone().map(|task| task.command)})}
                    />
                    <input type="text" name="data[working_dir]" placeholder="Work dir" required
                        value={selected_task.with_untracked(|sel_task| { sel_task.clone().map(|task| task.working_dir)})}
                    />
                    <input type="text" name="data[cron_expr]" placeholder="Cron Expression" required
                        value={selected_task.with_untracked(|sel_task| { sel_task.clone().map(|task| task.cron_expr)})}

                    />

                    <div>
                    <label name="data[is_active]">"Is Active"</label>
                    <select name="data[is_active]" required>
                        <option value="true"
                            prop:selected={selected_task.with_untracked(|sel_task| {
                                if let Some(task) = sel_task {
                                    task.is_active
                                } else {
                                    false
                                }
                            })}

                            >"True"</option>
                        <option value="false"
                            prop:selected={selected_task.with_untracked(|sel_task| {
                                if let Some(task) = sel_task {
                                    !task.is_active
                                } else {
                                    false
                                }
                            })}
                        >"False"</option>
                    </select>
                    </div>

                    <button type="submit" disabled=pending
                        prop:disabled= move || pending.get()
                    >
                        {move || if pending.get() { "Submitting..." } else { "Submit" }}
                    </button>

                </ActionForm>

            </div>
        </Modal>
    }
}
