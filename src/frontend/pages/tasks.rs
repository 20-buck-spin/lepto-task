use leptos::prelude::*;

use crate::frontend::components::tasks_comps::tasks_table::TasksTable;

#[component]
pub fn TasksPage() -> impl IntoView {
    return view! {
        <div class="page-block">
            <TasksTable/>
        </div>
    };
}
