use std::time::Duration;
use dioxus::prelude::*;
use crate::services::s3_data_fetcher::S3DataFetcher;

#[component]
pub fn ClientsCard() -> Element {
    let clients = use_signal(|| None as Option<u32>);
    let loading = use_signal(|| true);

    use_coroutine({
        let mut clients = clients.clone();
        let mut loading = loading.clone();
        move |_: UnboundedReceiver<u32>| async move {
            let buckets_count = if let Some(fetcher) = S3DataFetcher::from_db_account() {
                match fetcher.list_current_location(None, None).await {
                    Ok(buckets) => {
                        buckets.len() as u32
                    }
                    Err(e) => {
                        println!("error: {:?}", e);
                        0
                    }
                }
            } else {
                println!("S3DataFetcher::from_db_account() returned None - no default account?");
                0
            };
            clients.set(Some(buckets_count));
            loading.set(false);
        }
    });
    rsx!(
            div {
                class: "flex items-center p-4 bg-white rounded-lg shadow-xs dark:bg-gray-800",
                div {
                    class: "p-3 mr-4 text-orange-500 bg-orange-100 rounded-full dark:text-orange-100 dark:bg-orange-500",
                    svg {
                        class: "w-5 h-5",
                        fill: "currentColor",
                        view_box: "0 0 20 20",
                        path {
                            d: "M13 6a3 3 0 11-6 0 3 3 0 016 0zM18 8a2 2 0 11-4 0 2 2 0 014 0zM14 15a4 4 0 00-8 0v3h8v-3zM6 8a2 2 0 11-4 0 2 2 0 014 0zM16 18v-3a5.972 5.972 0 00-.75-2.906A3.005 3.005 0 0119 15v3h-3zM4.75 12.094A5.973 5.973 0 004 15v3H1v-3a3 3 0 013.75-2.906z"
                        }
                    }
                }
                div {
                    p {
                        class: "mb-2 text-sm font-medium text-gray-600 dark:text-gray-400",
                        "Total Buckets"
                    }
                    p {
                        class: "text-lg font-semibold text-gray-700 dark:text-gray-200",
                        if *loading.read() {
                            span { class: "loader", "Loading..." }
                        } else {
                            span { "{clients.read().unwrap_or_default()}" }
                        }
                    }
                }
            }
    )
}
