use std::time::Duration;

use crate::frontend::components::confirmation_modal::ConfirmationContext;
use crate::server_funcs::task_runs::cancel_run;
use leptos::task::spawn_local;
use leptos::{logging, prelude::*};

#[component]
pub fn CancelRunButton(run_id: i64, on_cancel: Callback<()>) -> impl IntoView {
    let confirm_ctx = expect_context::<ConfirmationContext>();

    let is_canceling = RwSignal::new(false);

    let on_cancel_click = move |_| {
        spawn_local(async move {
            let user_confirmed = confirm_ctx
                .get_confirmation(
                    "Cancel Task",
                    &format!("Are you sure you want to cancel task {}?", run_id),
                )
                .await;

            if user_confirmed {
                is_canceling.set(true);
                if let Err(err) = cancel_run(run_id).await {
                    logging::log!("Failed to cancel task. Err {}", err)
                }
                set_timeout(
                    move || {
                        is_canceling.set(false);
                        on_cancel.run(())
                    },
                    Duration::from_secs(3),
                );
            }
        });
    };

    view! {
        <button
            class="cancel-btn"
            on:click=on_cancel_click
            disabled=move || is_canceling.get()
        >
            {move || if is_canceling.get() { "Canceling..." } else { "Cancel" }}
        </button>
    }
}
