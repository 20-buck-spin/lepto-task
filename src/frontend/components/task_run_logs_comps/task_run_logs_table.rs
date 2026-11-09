use leptos::{logging, prelude::*};

use crate::{
    frontend::components::{
        empty_table::EmptyTable, pagination::Paginator, table_skeleton::TableSkeleton,
        task_run_logs_comps::raw_log_modal::RawLogModal,
    },
    models::task_models::RunLogFilters,
    server_funcs::task_run_logs::get_task_run_logs,
};

#[component]
pub fn TaskRunLogsTable(run_id: i64) -> impl IntoView {
    let current_page: RwSignal<i64> = RwSignal::new(1);
    let page_size: RwSignal<i64> = RwSignal::new(30);

    let show_raw_log_modal: RwSignal<bool> = RwSignal::new(false);
    let raw_log_for_display: RwSignal<String> = RwSignal::new(String::new());
    let log_filters = RwSignal::new(RunLogFilters {
        run_id: run_id,
        ..Default::default()
    });

    let logs = Resource::new(
        move || log_filters.get(),
        |filters| get_task_run_logs(filters),
    );

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        match RunLogFilters::from_event(&ev) {
            Err(err) => {
                logging::error!("Failed to parse new filters. Err {}", err);
                return;
            }
            Ok(new_filters) => {
                // limit and off set are defaulted
                log_filters.set(new_filters);
            }
        }
    };

    let on_page_change = Callback::new(move |(page_size, page_no): (i64, i64)| {
        log_filters.update(|filters| {
            filters.limit = page_size;
            filters.offset = (page_no - 1) * page_size;
        })
    });

    view! {
        <Show when=move || show_raw_log_modal.get()>
            <RawLogModal on_close = move || {show_raw_log_modal.set(false)} raw_log=raw_log_for_display />
        </Show>
        <div>

            <div class="table-header">
                <h3>"Run logs"</h3>
            </div>
            <div class="filter-form-container">
                <form on:submit=on_submit>
                    <input type="hidden" name="run_id" value=run_id/>
                    <div>
                        <label>Created from</label>
                        <input type="datetime-local" name="created_dt_from"/>
                    </div>
                    <div>
                        <label>Created to</label>
                        <input name="created_dt_to" type="datetime-local"/>
                    </div>
                    <div>
                        <label>Query</label>
                        <input name="query_string"/>
                    </div>
                    <div class="multi-select-filter">
                        <div>
                            <input type="checkbox" id="log-lvl-debug" name="levels[]" value="DEBUG"/>
                            <label for="log-lvl-debug"> Debug </label>
                        </div>
                        <div>
                            <input type="checkbox" id="log-lvl-info" name="levels[]" value="INFO"/>
                            <label for="log-lvl-info"> Info </label>
                        </div>
                        <div>
                            <input type="checkbox" id="log-lvl-warn" name="levels[]" value="WARN"/>
                            <label for="log-lvl-warn"> Warn</label>
                        </div>
                        <div>
                            <input type="checkbox" id="log-lvl-error" name="levels[]" value="ERROR"/>
                            <label for="log-lvl-error"> Error </label>
                        </div>
                        <div>
                            <input type="checkbox" id="log-lvl-critical" name="levels[]" value="CRITICAL"/>
                            <label for="log-lvl-critical"> Critical </label>
                        </div>
                    </div>
                    <input type="submit"/>
                </form>

            </div>


            <Suspense fallback= || view!{<TableSkeleton cols_len=5 rows_len=3/>}>
            <div class="table-outer-container">
                <div class="table-inner-container">

                    {
                        move || {
                                match logs.get() {
                                    Some(Ok(logs_list)) => {
                                        if logs_list.data.is_empty() {
                                            view! { <EmptyTable display_message="No logs found".to_string()/> }.into_any()
                                        } else {
                                            view!{
                                                <table>

                                                    <thead>
                                                        <tr>
                                                            <th>"ID"</th>
                                                            <th>"Created"</th>
                                                            <th>"Level"</th>
                                                            <th>"Message"</th>
                                                            <th>""</th>
                                                        </tr>
                                                    </thead>


                                                    <tbody>
                                                        <For
                                                        each=move || logs_list.data.clone()
                                                        key=|log| log.id
                                                        children= move |log| {
                                                            view!{
                                                                    <tr>
                                                                        <td>{log.id}</td>
                                                                        <td>{log.created_dt.to_string()}</td>
                                                                        <td>{log.level.to_string()}</td>
                                                                        <td>{log.message.clone()}</td>
                                                                        <td>{view!{
                                                                            <button on:click= move |_| {show_raw_log_modal.set(true);
                                                                                raw_log_for_display.set(log.raw_payload.clone())
                                                                            }
                                                                            >View Raw Logs</button>
                                                                        }}</td>
                                                                    </tr>
                                                                }
                                                        } />
                                                    </tbody>
                                                </table>
                                                <Paginator total_count=logs_list.total_count on_change=on_page_change current_page=current_page page_size=page_size/>
                                            }.into_any()
                                        }
                                    },
                                    Some(Err(err)) => view! { <span>"Got error " {err.to_string()}</span> }.into_any(),
                                    None => view! { <TableSkeleton rows_len=30 cols_len=5/> }.into_any(),}
                        }
                    }

                </div>
            </div>
            </Suspense>
        </div>
    }
}
