use leptos::prelude::*;

#[component]
pub fn ProfileDropDown() -> impl IntoView {
    let (is_open, set_is_open) = signal(false);
    view! {
        <div class="profile-icon-container">
            <div class="profile-icon-inner-container"
                on:pointerdown= move |_| {
                    set_is_open.update(|val| *val = !*val)
                }
            >
                <img src="/icons/profile.png" id="profile-icon"/>
            </div>

            <div class="profile-menu-outer-container"
                style:display=move || {if is_open.get() {"block"} else {"none"}}
            >
                <ul class="profile-menu">
                    <li><span>"Logout"</span></li>
                    <li><span>"Settings"</span></li>
                </ul>
            </div>
        </div>
    }
}
