use leptos::prelude::*;

#[component]
pub fn Modal(
    #[prop(into, default = "".into())] modal_title: Signal<String>,
    mut on_close: impl FnMut() + 'static,
    children: Children,
) -> impl IntoView {
    view! {
        <div class="modal-overlay">
            <div class="modal-outer-container">

                <div class = "modal-header">
                    {
                        move ||{
                            let title = modal_title.get();
                            if !title.is_empty() {
                                view!{ <h2>{title}</h2> }.into_any()
                            } else {
                                view!{}.into_any()
                            }
                        }
                    }
                    <button class="modal-close-btn"
                        on:click=move |_|{on_close()}
                    >"✕"</button>
                </div>

                <div class="modal-content">
                    {children()}
                </div>

            </div>
        </div>
    }
}
