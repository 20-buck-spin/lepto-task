use leptos::prelude::*;

#[component]
pub fn EmptyTable(display_message: String) -> impl IntoView {
    view! {
        <div class="empty-table table-inner-container">
            <p>{display_message}</p>
        </div>
    }
}
