use leptos::prelude::*;
use leptos_router::components::A;

use crate::frontend::components::profile_dropdown::ProfileDropDown;
#[component]
pub fn Header() -> impl IntoView {
    view! {
        <div class="header">
            <A href="/" attr:class="HeaderLogo">
                <img src="/logo.png" alt="Leptotask" id="headerLogo"/>
            </A>
            <ProfileDropDown/>
        </div>
    }
}
