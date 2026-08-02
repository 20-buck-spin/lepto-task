use crate::{
    frontend::{
        components::{
            confirmation_modal::{ConfirmationContext, ConfirmationModal},
            header::Header,
            nav_bar::NavBar,
        },
        pages::{
            dashboard::Dashboard, task_run_detail::TaskRunDetail, task_runs::TasksRunPage,
            tasks::TasksPage,
        },
    },
    ws::{pubsub_websocket, WsClientContext, WsMessage},
};
use futures::{channel::mpsc, StreamExt};
use leptos::{prelude::*, task::spawn_local};
use leptos_meta::{provide_meta_context, MetaTags, Stylesheet};
use leptos_router::{components::*, path};

pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <title> Leptotask </title>
                <AutoReload options=options.clone() />
                <HydrationScripts options/>
                <Stylesheet id="leptos" href="/pkg/leptotask.css"/>
                <MetaTags/>
            </head>
            <body>
                <App/>
            </body>
        </html>
    }
}

#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    let (tx, rx) = mpsc::channel::<Result<WsMessage, ServerFnError>>(10);
    let latest_message = RwSignal::new(None);
    let tx_stored = StoredValue::new(tx);
    let send_message = Callback::new(move |msg: WsMessage| {
        tx_stored.update_value(|tx_mut| {
            let _ = tx_mut.try_send(Ok(msg));
        });
    });

    provide_context(WsClientContext {
        latest_message,
        send: send_message,
    });

    provide_context(ConfirmationContext {
        request: RwSignal::new(None),
    });

    // Setup the listening thread only on the client side
    if cfg!(feature = "hydrate") || cfg!(target_arch = "wasm32") {
        spawn_local(async move {
            match pubsub_websocket(rx.into()).await {
                Ok(mut incoming_stream) => {
                    while let Some(msg_result) = incoming_stream.next().await {
                        match msg_result {
                            Ok(msg) => latest_message.set(Some(msg)),
                            Err(e) => leptos::logging::error!("Incoming WS Error: {:?}", e),
                        }
                    }
                }
                Err(e) => leptos::logging::error!("Failed to connect WS: {:?}", e),
            }
        });
    }

    view! {
        <ConfirmationModal/>
        <Router>
            <Header/>
                <Routes fallback = || "Not Found">
                    <Route path=path!("/") view=Dashboard />
                    <Route path=path!("/task_runs") view=TasksRunPage />
                    <Route path=path!("/task_runs/:task_run_id") view=TaskRunDetail/>
                    <Route path=path!("/tasks") view=TasksPage />
               </Routes>
            <NavBar/>
        </Router>
    }
}
