use futures::channel::oneshot;
use leptos::prelude::*;

pub struct ConfirmationRequest {
    pub title: String,
    pub description: String,
    pub responder: oneshot::Sender<bool>,
}

#[derive(Clone, Copy)]
pub struct ConfirmationContext {
    pub request: RwSignal<Option<ConfirmationRequest>>,
}

impl ConfirmationContext {
    pub fn new() -> Self {
        Self {
            request: RwSignal::new(None),
        }
    }

    pub async fn get_confirmation(&self, title: &str, message: &str) -> bool {
        let (tx, rx) = oneshot::channel();
        self.request.set(Some(ConfirmationRequest {
            title: title.to_string(),
            description: message.to_string(),
            responder: tx,
        }));
        rx.await.unwrap_or(false)
    }
}

#[component]
pub fn ConfirmationModal() -> impl IntoView {
    let confirm_ctx = expect_context::<ConfirmationContext>();

    let resolve = move |result: bool| {
        confirm_ctx.request.update(|req_option| {
            if let Some(req) = req_option.take() {
                let _ = req.responder.send(result);
            }
        })
    };
    view! {
        {move || {
            confirm_ctx.request.with(|req| {
                req.as_ref().map(|r| {
                    view! {
                        <div class="confirm-modal-backdrop">
                            <div class="confirm-modal-box">
                                <div class="confirmation-modal-header">
                                    <h3 class="confirm-modal-title">{r.title.clone()}</h3>
                                    <button on:click=move |_| resolve(false)>X</button>
                                </div>
                                <p class="confirm-modal-description">{r.description.clone()}</p>
                                <div class="confirm-modal-actions">
                                    <button
                                        class="confirm-modal-btn confirm-modal-btn-cancel"
                                        on:click=move |_| resolve(false)
                                    >
                                        "Cancel"
                                    </button>
                                    <button
                                        class="confirm-modal-btn confirm-modal-btn-confirm"
                                        on:click=move |_| resolve(true)
                                    >
                                        "Confirm"
                                    </button>
                                </div>
                            </div>
                        </div>
                    }
                })
            })
        }}
    }
}
