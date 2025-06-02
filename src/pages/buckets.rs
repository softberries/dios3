use std::time::Duration;
use dioxus::prelude::*;
use dioxus::hooks::{use_coroutine, use_signal};
use crate::components::{AccountCard, BucketModal, ClientsCard, ContactsCard, SalesCard};
use tokio::task::spawn_blocking;
use crate::components::github_star_action::GithubStarAction;
use crate::model::bucket::Bucket;
use crate::services::s3_data_fetcher::S3DataFetcher;
use crate::utils::CURRENT_ACCOUNT;

const S3_IMG: Asset = asset!("/assets/aws_logo.png");

async fn list_buckets() -> Vec<Bucket> {
    if let Some(fetcher) = S3DataFetcher::from_db_account() {
        match fetcher.list_current_location(None, None).await {
            Ok(buckets) => buckets.iter().map(|b| -> Bucket { 
                let region_clone = b.region.as_ref().map(|r| r.to_string());
                Bucket { name: b.name.clone(), region: region_clone } 
            }).collect(),
            Err(_) => Vec::new(),
        }
    } else {
        Vec::new()
    }
}

fn delete_bucket(bucket_name: &str) -> () {
    // TODO: Implement actual bucket deletion
    println!("Deleting bucket with name: {}", bucket_name);
}

/// Home page
#[component]
pub fn Buckets() -> Element {
    let mut show_modal = use_signal(|| false);
    let buckets = use_signal(|| Vec::<Bucket>::new());
    let selected_bucket = use_signal(|| None as Option<Bucket>);
    let mut refresh_buckets = use_signal(|| false);
    let mut bucket_to_delete = use_signal(|| None as Option<Bucket>);

    use_effect(move || {
        let mut buckets = buckets.clone();
        spawn(async move {
            let data = list_buckets().await;
            buckets.set(data);
        });
    });

    use_effect(move || {
        if *refresh_buckets.read() {
            let mut buckets = buckets.clone();
            spawn(async move {
                let data = list_buckets().await;
                buckets.set(data);
                refresh_buckets.set(false);
            });
        }
    });

    rsx!(
        if *show_modal.read() {
                    BucketModal {
                        show_modal: show_modal.clone(),
                        selected_bucket: selected_bucket.clone(),
                        refresh_buckets: refresh_buckets.clone(),
                    }
                },
        if let Some(bck) = bucket_to_delete.read().as_ref() {
                div { class: "fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center",
                    div { class: "bg-white dark:bg-gray-800 p-6 rounded shadow",
                        h2 { class: "text-lg font-bold mb-4", "Confirm Delete" }
                        p { "Are you sure you want to delete bucket: ", {bck.name.clone()}, "?" }
                        div { class: "flex justify-end space-x-2",
                            button {
                                class: "px-4 py-2 bg-gray-300 rounded hover:bg-gray-400",
                                onclick: move |_| bucket_to_delete.set(None),
                                "Cancel"
                            }
                            button {
                                class: "px-4 py-2 bg-red-600 text-white rounded hover:bg-red-700",
                                onclick: {
                                    let mut bucket_to_delete = bucket_to_delete.clone();
                                    let mut refresh_buckets = refresh_buckets.clone();
                                    let name = bck.name.clone();
                                    move |_| {
                                        let name = name.clone();
                                        spawn_blocking(move || delete_bucket(&name));
                                        bucket_to_delete.set(None);
                                        refresh_buckets.set(true);
                                    }
                                }, 
                                "Delete"
                            }
                        }
                    }
                }
            },
        main { class: "h-full overflow-y-auto",
                div { class: "container px-6 mx-auto grid",
                    div { class: "flex items-center justify-between mt-6 mb-5",
                        h2 {
                            class: "text-2xl font-semibold text-gray-700 dark:text-gray-200",
                            "Buckets"
                        },
                        button {
                            class: "px-4 py-2 mr-4 text-sm font-medium text-white bg-purple-600 rounded-lg hover:bg-purple-700 focus:outline-none focus:ring",
                            onclick: move |_| show_modal.set(true),
                            "New Bucket"
                        }
                    }
                    GithubStarAction {},
                    BucketsTable { buckets: buckets.read().clone(), selected_bucket: selected_bucket.clone(), show_modal: show_modal.clone(), bucket_to_delete: bucket_to_delete.clone()}
                }
            }
    )
}

#[component]
fn BucketsTable(buckets: Vec<Bucket>, selected_bucket: Signal<Option<Bucket>>, show_modal: Signal<bool>, bucket_to_delete: Signal<Option<Bucket>>) -> Element {
    rsx! {
    div { class: "w-full overflow-hidden rounded-lg shadow-xs",
        div { class: "w-full overflow-x-auto",
            table { class: "w-full whitespace-no-wrap",
                thead {
                    tr {
                        class: "text-xs font-semibold tracking-wide text-left text-gray-500 uppercase border-b dark:border-gray-700 bg-gray-50 dark:text-gray-400 dark:bg-gray-800",
                        th { class: "px-4 py-3", "Name" }
                        th { class: "px-4 py-3", "Region" }
                        th { class: "px-4 py-3", "Actions" }
                    }
                }
                tbody { class: "bg-white divide-y dark:divide-gray-700 dark:bg-gray-800",
                    {buckets.into_iter().map(|bck| {
                        let bck_for_edit = bck.clone();
                        let bck_for_delete = bck.clone();
                        let mut selected_bucket= selected_bucket.clone();
                        let mut show_modal = show_modal.clone();
                        rsx!(
                        tr { class: "text-gray-700 dark:text-gray-400",
                                td { class: "px-4 py-3",
                                    div { class: "flex items-center text-sm",
                                        div { class: "relative hidden w-8 h-8 mr-3 rounded-full md:block",
                                            img {
                                                class: "object-cover w-full h-full rounded-full",
                                                src: "{S3_IMG}",
                                                alt: "",
                                                loading: "lazy"
                                            }
                                            div { class: "absolute inset-0 rounded-full shadow-inner", aria_hidden: "true" }
                                        }
                                        div {
                                            p { class: "font-semibold", "{bck.name}" }
                                        }
                                    }
                                }
                            td { class: "px-4 py-3 text-sm", "{bck.region.clone().unwrap_or_default()}" }
                            td { class: "px-4 py-3 space-x-2",
                        button {
                            class: "px-2 py-1 text-sm text-white bg-blue-500 rounded hover:bg-blue-600 focus:outline-none",
                            onclick: {
                                let mut selected_bucket= selected_bucket.clone();
                                let mut show_modal = show_modal.clone();
                                move |_| {
                                    selected_bucket.set(Some(bck_for_edit.clone()));
                                    show_modal.set(true);
                                }
                            },
                            "Edit"
                        }
                                button {
                                    class: "px-2 py-1 text-sm text-white bg-red-500 rounded hover:bg-red-600 focus:outline-none",
                                    onclick: {
                                        let mut bucket_to_delete = bucket_to_delete.clone();
                                        // let mut refresh_buckets = refresh_buckets.clone();
                                        let name = bck.name.clone();
                                        move |_| {
                                            let name = name.clone();
                                            spawn_blocking(move || delete_bucket(&name));
                                            bucket_to_delete.set(None);
                                            // refresh_buckets.set(true);
                                        }
                                    },
                                    "Delete"
                                }
                            }
                        }
                    )})}
                }
            }
        }

        // Pagination
        div {
            class: "grid px-4 py-3 text-xs font-semibold tracking-wide text-gray-500 uppercase border-t dark:border-gray-700 bg-gray-50 sm:grid-cols-9 dark:text-gray-400 dark:bg-gray-800",
            span { class: "flex items-center col-span-3", "Showing 21-30 of 100" }
            span { class: "col-span-2" }
            span { class: "flex col-span-4 mt-2 sm:mt-auto sm:justify-end",
                nav { aria_label: "Table navigation",
                    ul { class: "inline-flex items-center",
                        li {
                            button {
                                class: "px-3 py-1 rounded-md rounded-l-lg focus:outline-none focus:shadow-outline-purple",
                                aria_label: "Previous",
                                svg {
                                    class: "w-4 h-4 fill-current",
                                    view_box: "0 0 20 20",
                                    path {
                                        d: "M12.707 5.293a1 1 0 010 1.414L9.414 10l3.293 3.293a1 1 0 01-1.414 1.414l-4-4a1 1 0 010-1.414l4-4a1 1 0 011.414 0z",
                                        clip_rule: "evenodd",
                                        fill_rule: "evenodd"
                                    }
                                }
                            }
                        }
                        li { button { class: "px-3 py-1 rounded-md focus:outline-none focus:shadow-outline-purple", "1" } }
                        li { button { class: "px-3 py-1 rounded-md focus:outline-none focus:shadow-outline-purple", "2" } }
                        li {
                            button {
                                class: "px-3 py-1 text-white transition-colors duration-150 bg-purple-600 border border-r-0 border-purple-600 rounded-md focus:outline-none focus:shadow-outline-purple",
                                "3"
                            }
                        }
                        li { button { class: "px-3 py-1 rounded-md focus:outline-none focus:shadow-outline-purple", "4" } }
                        li { span { class: "px-3 py-1", "..." } }
                        li { button { class: "px-3 py-1 rounded-md focus:outline-none focus:shadow-outline-purple", "8" } }
                        li { button { class: "px-3 py-1 rounded-md focus:outline-none focus:shadow-outline-purple", "9" } }
                        li {
                            button {
                                class: "px-3 py-1 rounded-md rounded-r-lg focus:outline-none focus:shadow-outline-purple",
                                aria_label: "Next",
                                svg {
                                    class: "w-4 h-4 fill-current",
                                    view_box: "0 0 20 20",
                                    path {
                                        d: "M7.293 14.707a1 1 0 010-1.414L10.586 10 7.293 6.707a1 1 0 011.414-1.414l4 4a1 1 0 010 1.414l-4 4a1 1 0 01-1.414 0z",
                                        clip_rule: "evenodd",
                                        fill_rule: "evenodd"
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
}