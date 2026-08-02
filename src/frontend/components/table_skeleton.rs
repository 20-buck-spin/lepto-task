use leptos::prelude::*;

#[component]
pub fn TableSkeleton(cols_len: u16, rows_len: u16) -> impl IntoView {
    view! {
        <div class="table-outer-container">
            <div class="table-inner-container">
            <table>
                <thead>
                <tr>
                    {
                        (0..cols_len).into_iter().map(|_| {
                            view!{
                                <th>
                                    <div class="skeleton-header-el"></div>
                                </th>
                            }
                        }).collect::<Vec<_>>()
                    }
                </tr>
                </thead>
                <tbody>
                {
                    (0..rows_len).into_iter().map(|_| {
                        view!{
                            <tr>
                            {
                                (0..cols_len).into_iter().map(|_| {
                                    view!{
                                        <td>
                                            <div class="skeleton-row-el"></div>
                                        </td>
                                    }
                                }).collect::<Vec<_>>()

                            }
                            </tr>
                        }
                    }).collect::<Vec<_>>()
                }
                </tbody>
            </table>
        </div>
        </div>
    }
}
