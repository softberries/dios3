use std::time::Duration;
use dioxus::prelude::*;
use dioxus::hooks::{use_coroutine, use_signal};
use crate::components::{AccountCard, BucketModal, ClientsCard, ContactsCard, SalesCard};
use tokio::task::spawn_blocking;
use crate::components::github_star_action::GithubStarAction;
use crate::model::bucket::Bucket;
use crate::services::s3_data_fetcher::S3DataFetcher;
use crate::utils::CURRENT_ACCOUNT;

const BUCKET_ICON: Asset = asset!("/assets/bucket_icon.png");

async fn list_buckets(page: Option<usize>, page_size: Option<usize>) -> (Vec<Bucket>, usize) {
    if let Some(fetcher) = S3DataFetcher::from_db_account() {
        match fetcher.list_buckets(page, page_size).await {
            Ok((s3_buckets, total_count)) => {
                // Return buckets immediately without regions
                let buckets = s3_buckets.iter().map(|s3_bucket| {
                    Bucket {
                        name: s3_bucket.name.clone(),
                        region: None, // Will be populated asynchronously
                    }
                }).collect();
                (buckets, total_count)
            }
            Err(e) => {
                println!("Failed to list buckets: {}", e);
                (Vec::new(), 0)
            }
        }
    } else {
        println!("No S3DataFetcher available - no default account configured");
        (Vec::new(), 0)
    }
}

async fn fetch_bucket_regions(buckets_signal: Signal<Vec<Bucket>>) {
    if let Some(fetcher) = S3DataFetcher::from_db_account() {
        let current_buckets = buckets_signal.read().clone();
        
        for bucket in current_buckets.iter() {
            if bucket.region.is_none() {
                let bucket_name = bucket.name.clone();
                let fetcher_clone = fetcher.clone();
                let mut buckets_signal_clone = buckets_signal.clone();
                
                spawn(async move {
                    match fetcher_clone.get_bucket_location(&bucket_name).await {
                        Ok(region) => {
                            println!("Got region '{}' for bucket '{}'", region, bucket_name);
                            let mut current_buckets = buckets_signal_clone.read().clone();
                            // Find bucket by name instead of using index
                            if let Some(bucket_to_update) = current_buckets.iter_mut().find(|b| b.name == bucket_name) {
                                bucket_to_update.region = Some(region);
                                buckets_signal_clone.set(current_buckets);
                            }
                        }
                        Err(e) => {
                            println!("Failed to get region for bucket '{}': {}", bucket_name, e);
                            let mut current_buckets = buckets_signal_clone.read().clone();
                            // Find bucket by name instead of using index
                            if let Some(bucket_to_update) = current_buckets.iter_mut().find(|b| b.name == bucket_name) {
                                bucket_to_update.region = Some("Error".to_string());
                                buckets_signal_clone.set(current_buckets);
                            }
                        }
                    }
                });
            }
        }
    }
}

async fn delete_bucket(bucket_name: String) -> Result<(), String> {
    if let Some(fetcher) = S3DataFetcher::from_db_account() {
        match fetcher.delete_data(true, None, bucket_name.clone(), false).await {
            Ok(None) => {
                println!("Bucket '{}' deleted successfully", bucket_name);
                Ok(())
            }
            Ok(Some(error_msg)) => {
                println!("Failed to delete bucket: {}", error_msg);
                Err(error_msg)
            }
            Err(e) => {
                let error_msg = format!("Error deleting bucket: {}", e);
                println!("{}", error_msg);
                Err(error_msg)
            }
        }
    } else {
        let error_msg = "No default account configured. Please set up an AWS account first.".to_string();
        println!("{}", error_msg);
        Err(error_msg)
    }
}

/// Home page
#[component]
pub fn Buckets() -> Element {
    let mut show_modal = use_signal(|| false);
    let buckets = use_signal(|| Vec::<Bucket>::new());
    let mut refresh_buckets = use_signal(|| false);
    let mut bucket_to_delete = use_signal(|| None as Option<Bucket>);
    let mut current_page = use_signal(|| 0usize);
    let mut page_size = use_signal(|| 20usize);
    let mut total_buckets = use_signal(|| 0usize);

    use_effect(move || {
        let mut buckets_signal = buckets.clone();
        let page = current_page.read().clone();
        let size = page_size.read().clone();
        spawn(async move {
            let (bucket_data, total) = list_buckets(Some(page), Some(size)).await;
            buckets_signal.set(bucket_data);
            total_buckets.set(total);
            // Fetch regions asynchronously after buckets are loaded
            fetch_bucket_regions(buckets_signal).await;
        });
    });

    use_effect(move || {
        if *refresh_buckets.read() {
            let mut buckets_signal = buckets.clone();
            let page = current_page.read().clone();
            let size = page_size.read().clone();
            spawn(async move {
                let (bucket_data, total) = list_buckets(Some(page), Some(size)).await;
                buckets_signal.set(bucket_data);
                total_buckets.set(total);
                refresh_buckets.set(false);
                // Fetch regions asynchronously after buckets are refreshed
                fetch_bucket_regions(buckets_signal).await;
            });
        }
    });

    // Effect to reload buckets when page or page_size changes
    use_effect(move || {
        let mut buckets_signal = buckets.clone();
        let page = current_page.read().clone();
        let size = page_size.read().clone();
        spawn(async move {
            let (bucket_data, total) = list_buckets(Some(page), Some(size)).await;
            buckets_signal.set(bucket_data);
            total_buckets.set(total);
            fetch_bucket_regions(buckets_signal).await;
        });
    });

    rsx!(
        if *show_modal.read() {
                    BucketModal {
                        show_modal: show_modal.clone(),
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
                                    let bucket_name = bck.name.clone();
                                    let mut bucket_to_delete = bucket_to_delete.clone();
                                    let mut refresh_buckets = refresh_buckets.clone();
                                    move |_| {
                                        let name = bucket_name.clone();
                                        spawn(async move {
                                            match delete_bucket(name).await {
                                                Ok(()) => {
                                                    bucket_to_delete.set(None);
                                                    refresh_buckets.set(true);
                                                }
                                                Err(err) => {
                                                    // TODO: Show error message to user
                                                    println!("Delete failed: {}", err);
                                                    bucket_to_delete.set(None);
                                                }
                                            }
                                        });
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
                        div { class: "flex items-center space-x-4",
                            div { class: "flex items-center space-x-2",
                                label { class: "text-sm text-gray-600 dark:text-gray-400", "Page size:" }
                                select {
                                    class: "px-3 py-1 text-sm border rounded-md dark:bg-gray-700 dark:text-white",
                                    value: "{page_size}",
                                    onchange: move |e| {
                                        if let Ok(new_size) = e.value().parse::<usize>() {
                                            page_size.set(new_size);
                                            current_page.set(0); // Reset to first page
                                        }
                                    },
                                    option { value: "10", "10" }
                                    option { value: "20", "20" }
                                    option { value: "50", "50" }
                                    option { value: "100", "100" }
                                }
                            }
                            button {
                                class: "px-4 py-2 ml-6 text-sm font-medium text-white bg-purple-600 rounded-lg hover:bg-purple-700 focus:outline-none focus:ring",
                                onclick: move |_| show_modal.set(true),
                                "New Bucket"
                            }
                        }
                    }
                    GithubStarAction {},
                    BucketsTable { 
                        buckets: buckets.read().clone(), 
                        bucket_to_delete: bucket_to_delete.clone(), 
                        refresh_buckets: refresh_buckets.clone(),
                        current_page: current_page.clone(),
                        page_size: page_size.clone(),
                        total_buckets: total_buckets.clone(),
                    }
                }
            }
    )
}

#[component]
fn BucketsTable(buckets: Vec<Bucket>, bucket_to_delete: Signal<Option<Bucket>>, refresh_buckets: Signal<bool>, current_page: Signal<usize>, page_size: Signal<usize>, total_buckets: Signal<usize>) -> Element {
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
                        let bck_for_delete = bck.clone();
                        rsx!(
                        tr { class: "text-gray-700 dark:text-gray-400",
                                td { class: "px-4 py-3",
                                    div { class: "flex items-center text-sm",
                                        div { class: "relative hidden w-8 h-8 mr-3 rounded-full md:block",
                                            img {
                                                class: "object-cover w-full h-full rounded-full",
                                                src: "{BUCKET_ICON}",
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
                            td { class: "px-4 py-3 text-sm", 
                                if let Some(region) = &bck.region {
                                    "{region}"
                                } else {
                                    span { 
                                        class: "inline-flex items-center text-gray-500",
                                        svg {
                                            class: "animate-spin -ml-1 mr-2 h-4 w-4 text-gray-500",
                                            fill: "none",
                                            view_box: "0 0 24 24",
                                            circle {
                                                class: "opacity-25",
                                                cx: "12",
                                                cy: "12",
                                                r: "10",
                                                stroke: "currentColor",
                                                stroke_width: "4"
                                            }
                                            path {
                                                class: "opacity-75",
                                                fill: "currentColor",
                                                d: "M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
                                            }
                                        }
                                        "Loading..."
                                    }
                                }
                            }
                            td { class: "px-4 py-3 space-x-2",
                                button {
                                    class: "px-2 py-1 text-sm text-white bg-red-500 rounded hover:bg-red-600 focus:outline-none",
                                    onclick: {
                                        let mut bucket_to_delete = bucket_to_delete.clone();
                                        move |_| {
                                            bucket_to_delete.set(Some(bck_for_delete.clone()));
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
            span { class: "flex items-center col-span-3", 
                {
                    let current = *current_page.read();
                    let size = *page_size.read();
                    let total = *total_buckets.read();
                    let start = if total > 0 { current * size + 1 } else { 0 };
                    let end = std::cmp::min((current + 1) * size, total);
                    format!("Showing {}-{} of {}", start, end, total)
                }
            }
            span { class: "col-span-2" }
            span { class: "flex col-span-4 mt-2 sm:mt-auto sm:justify-end",
                nav { aria_label: "Table navigation",
                    ul { class: "inline-flex items-center",
                        li {
                            button {
                                class: "px-3 py-1 rounded-md rounded-l-lg focus:outline-none focus:shadow-outline-purple disabled:opacity-50 disabled:cursor-not-allowed",
                                aria_label: "Previous",
                                disabled: *current_page.read() == 0,
                                onclick: move |_| {
                                    let current = *current_page.read();
                                    if current > 0 {
                                        current_page.set(current - 1);
                                    }
                                },
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
                        li { 
                            span { class: "px-3 py-1", 
                                "Page {current_page.read().clone() + 1} of {((total_buckets.read().clone() as f64 / page_size.read().clone() as f64).ceil() as usize).max(1)}"
                            } 
                        }
                        li {
                            button {
                                class: "px-3 py-1 rounded-md rounded-r-lg focus:outline-none focus:shadow-outline-purple disabled:opacity-50 disabled:cursor-not-allowed",
                                aria_label: "Next",
                                disabled: {
                                    let current = *current_page.read();
                                    let size = *page_size.read();
                                    let total = *total_buckets.read();
                                    let max_page = if total > 0 { (total - 1) / size } else { 0 };
                                    current >= max_page
                                },
                                onclick: move |_| {
                                    let current = *current_page.read();
                                    let size = *page_size.read();
                                    let total = *total_buckets.read();
                                    let max_page = if total > 0 { (total - 1) / size } else { 0 };
                                    if current < max_page {
                                        current_page.set(current + 1);
                                    }
                                },
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