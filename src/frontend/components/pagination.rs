use leptos::prelude::*;

#[component]
pub fn Paginator(
    total_count: i64,
    on_change: Callback<(i64, i64)>,
    current_page: RwSignal<i64>,
    page_size: RwSignal<i64>,
) -> impl IntoView {
    let total_pages = Memo::new(move |_| {
        if total_count == 0 {
            1
        } else {
            (total_count as f64 / page_size.get() as f64).ceil() as i64
        }
    });

    let on_page_size_change = {
        let on_change = on_change.clone();
        move |ev| {
            let Ok(new_page_size) = event_target_value(&ev).parse::<i64>() else {
                return;
            };

            page_size.set(new_page_size);
            current_page.set(1);

            on_change.run((new_page_size, 1));
        }
    };

    let on_page_number_change = {
        let on_change = on_change.clone();

        move |ev| {
            let Ok(new_page) = event_target_value(&ev).parse::<i64>() else {
                return;
            };
            current_page.set(new_page);
            on_change.run((page_size.get(), new_page))
        }
    };

    let on_prev = {
        let on_change = on_change.clone();

        move |_| {
            let page = current_page.get();

            if page <= 1 {
                return;
            }

            let new_page = page - 1;
            current_page.set(new_page);

            on_change.run((page_size.get(), new_page));
        }
    };

    let on_next = {
        let on_change = on_change.clone();

        move |_| {
            let page = current_page.get();
            let total_pages = total_pages.get();

            if page >= total_pages {
                return;
            }

            let new_page = page + 1;
            current_page.set(new_page);

            on_change.run((page_size.get(), new_page));
        }
    };

    view! {
        <div class="paginator-container">
            <div class="page-size-selector">
                <label for="page-size">Items per page:</label>
                <select id="page-size"
                        on:change=on_page_size_change
                        prop:value=page_size>
                    <option value=10>10</option>
                    <option value=20>20</option>
                    <option value=30>30</option>
                    <option value=50>50</option>
                    <option value=100>100</option>
                </select>
            </div>

            <div class="pagination-controls">
                <button on:click=on_prev disabled={move || current_page.get() <= 1}>
                    "Previous"
                </button>

                <span>
                    <input
                        type="number"
                        min="1"
                        max=total_pages
                        prop:value=current_page
                        on:change=on_page_number_change
                        class="current-page-input"
                    />
                    "of "{total_pages}
                </span>

                <button on:click=on_next disabled={move || current_page.get() >= total_pages.get()}>
                    "Next"
                </button>
            </div>
        </div>
    }
}
