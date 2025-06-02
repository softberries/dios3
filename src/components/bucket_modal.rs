use dioxus::prelude::*;
use once_cell::sync::Lazy;
use std::sync::Mutex;
use crate::utils::DB;
use tokio::task::spawn_blocking;
use crate::model::bucket::Bucket;

#[derive(Props, Clone, PartialEq)]
pub struct BucketModalProps {
    show_modal: Signal<bool>,
    selected_bucket: Signal<Option<Bucket>>,
    refresh_buckets: Signal<bool>,
}

async fn save_bucket(name: &str, region: &str) {
    println!("Saving bucket with name: {} and region: {}", name, region);
}

#[component]
pub fn BucketModal(mut props: BucketModalProps) -> Element {
    let bucket = props.selected_bucket.read().clone();
    let mut bucket_name = use_signal(|| bucket.as_ref().map(|a| a.name.clone()).unwrap_or_default());
    let mut region = use_signal(|| bucket.as_ref().map(|a| a.region.as_ref().map(|r| r.clone()).unwrap_or_default()).unwrap_or_default());
    rsx! {
        div {
            class: "fixed inset-0 z-50 w-screen h-screen flex items-center justify-center bg-black bg-opacity-50",
            onclick: move |_| props.show_modal.set(false),
            div {
                class: "bg-white dark:bg-gray-800 rounded-lg p-6 shadow-xl w-full max-w-md",
                onclick: move |e| e.stop_propagation(), // prevent click from closing the modal

                h2 { class: "text-xl font-bold mb-4 text-gray-900 dark:text-gray-100", if bucket.is_some() { "Edit Bucket" } else { "New Bucket" } }

                form {
                    class: "space-y-4",
                    onsubmit: move |evt| {
                        evt.prevent_default();

                        let name = bucket_name.read().clone();
                        let region = region.read().clone();

                        spawn_blocking(move || {
                            save_bucket(&name, &region);
                        });
                        props.refresh_buckets.set(true);
                        props.show_modal.set(false);
                    },
                    div {
                        label { class: "block text-sm font-medium text-gray-700 dark:text-gray-300", "Account Name" }
                        input {
                            class: "w-full px-3 py-2 border rounded-md dark:bg-gray-700 dark:text-white",
                            r#type: "text",
                            placeholder: "Enter bucket name",
                            value: "{bucket_name}",
                            oninput: move |e| bucket_name.set(e.value().clone()),
                        }
                    }
                    div {
                        label { class: "block text-sm font-medium text-gray-700 dark:text-gray-300", "Default Region" }
                        select {
                            class: "w-full px-3 py-2 border rounded-md dark:bg-gray-700 dark:text-white",
                            value: "{region}",
                            oninput: move |e| region.set(e.value().clone()),
                            option { value: "", "Select a region" }
                            option { value: "us-east-1", "US East (N. Virginia)" }
                            option { value: "us-west-1", "US West (N. California)" }
                            option { value: "us-west-2", "US West (Oregon)" }
                            option { value: "eu-west-1", "EU (Ireland)" }
                            option { value: "eu-central-1", "EU (Frankfurt)" }
                            option { value: "ap-southeast-1", "Asia Pacific (Singapore)" }
                            option { value: "ap-northeast-1", "Asia Pacific (Tokyo)" }
                            option { value: "ap-southeast-2", "Asia Pacific (Sydney)" }
                            option { value: "ap-northeast-2", "Asia Pacific (Seoul)" }
                            option { value: "sa-east-1", "South America (São Paulo)" }
                            option { value: "ca-central-1", "Canada (Central)" }
                        }
                    }
                    div {
                        button {
                            class: "bg-purple-600 text-white px-4 py-2 rounded hover:bg-purple-700",
                            r#type: "submit",
                            "Save"
                        }
                    }
                }
            }
        }
    }
}