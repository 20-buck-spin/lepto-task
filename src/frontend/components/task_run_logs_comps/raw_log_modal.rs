use crate::frontend::components::modal::Modal;
use leptos::prelude::*;

#[component]
pub fn RawLogModal(on_close: impl FnMut() + 'static, raw_log: RwSignal<String>) -> impl IntoView {
    view! {
        <Modal modal_title="Raw log data" on_close=on_close>
            <div>
                {raw_log}
            </div>
        </Modal>
    }
}
