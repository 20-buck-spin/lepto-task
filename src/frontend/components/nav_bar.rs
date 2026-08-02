use leptos::prelude::*;
use leptos_router::components::A;

#[component]
pub fn NavBar() -> impl IntoView {
    view! {
        <nav class="navBar">
            <ul class="navBarList">
                <li class="navListItem">
                    <A href="/">
                        <img src="/icons/dashboard_icon.png" alt="Star" width="24" height="24"/>
                        <span>"Dashboard"</span>
                    </A>
                </li>
                <li class="navListItem">
                    <A href="/task_runs">
                        <img src="/icons/tasks_icon.png" alt="Star" width="24" height="24"/>
                        <span>"Runs"</span>
                    </A>
                </li>
                <li class="navListItem">
                    <A href="/tasks">
                        <img src="/icons/users_icon.png" alt="Star" width="24" height="24"/>
                        <span>"Tasks"</span>
                    </A>
                </li>
            </ul>
        </nav>
    }
}
