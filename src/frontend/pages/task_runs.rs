use leptos::prelude::*;

use crate::frontend::components::runs_comps::runs_table::RunsTable;

#[component]
pub fn TasksRunPage() -> impl IntoView {
    view! {
        <div class="page-block">
            <RunsTable/>
        </div>
    }
}
