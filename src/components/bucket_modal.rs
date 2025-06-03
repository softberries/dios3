use dioxus::prelude::*;
use once_cell::sync::Lazy;
use std::sync::Mutex;
use crate::utils::DB;
use tokio::task::spawn_blocking;
use crate::model::bucket::Bucket;
use crate::services::s3_data_fetcher::S3DataFetcher;

#[derive(Props, Clone, PartialEq)]
pub struct BucketModalProps {
    show_modal: Signal<bool>,
    refresh_buckets: Signal<bool>,
}

async fn save_bucket(name: &str, region: &str) -> Result<(), String> {
    if let Some(fetcher) = S3DataFetcher::from_db_account() {
        match fetcher.create_bucket(name.to_string(), region.to_string()).await {
            Ok(None) => {
                println!("Bucket '{}' created successfully in region '{}'", name, region);
                Ok(())
            }
            Ok(Some(error_msg)) => {
                println!("Failed to create bucket: {}", error_msg);
                Err(error_msg)
            }
            Err(e) => {
                let error_msg = format!("Error creating bucket: {}", e);
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

#[component]
pub fn BucketModal(mut props: BucketModalProps) -> Element {
    let mut bucket_name = use_signal(|| String::new());
    let mut region = use_signal(|| String::new());
    let mut error_message = use_signal(|| None as Option<String>);
    let mut is_saving = use_signal(|| false);
    
    rsx! {
        div {
            class: "fixed inset-0 z-50 w-screen h-screen flex items-center justify-center bg-black bg-opacity-50",
            onclick: move |_| props.show_modal.set(false),
            div {
                class: "bg-white dark:bg-gray-800 rounded-lg p-6 shadow-xl w-full max-w-md",
                onclick: move |e| e.stop_propagation(), // prevent click from closing the modal

                h2 { class: "text-xl font-bold mb-4 text-gray-900 dark:text-gray-100", "New Bucket" }

                if let Some(error) = error_message.read().as_ref() {
                    div {
                        class: "mb-4 p-3 bg-red-100 border border-red-400 text-red-700 rounded",
                        "{error}"
                    }
                }

                form {
                    class: "space-y-4",
                    onsubmit: move |evt| {
                        evt.prevent_default();
                        
                        let name = bucket_name.read().clone();
                        let region = region.read().clone();
                        
                        if name.trim().is_empty() {
                            error_message.set(Some("Bucket name is required".to_string()));
                            return;
                        }
                        
                        if region.trim().is_empty() {
                            error_message.set(Some("Region is required".to_string()));
                            return;
                        }

                        error_message.set(None);
                        is_saving.set(true);
                        
                        spawn(async move {
                            match save_bucket(&name, &region).await {
                                Ok(()) => {
                                    props.refresh_buckets.set(true);
                                    props.show_modal.set(false);
                                }
                                Err(err) => {
                                    error_message.set(Some(err));
                                    is_saving.set(false);
                                }
                            }
                        });
                    },
                    div {
                        label { class: "block text-sm font-medium text-gray-700 dark:text-gray-300", "Bucket Name" }
                        input {
                            class: "w-full px-3 py-2 border rounded-md dark:bg-gray-700 dark:text-white",
                            r#type: "text",
                            placeholder: "Enter bucket name",
                            value: "{bucket_name}",
                            oninput: move |e| bucket_name.set(e.value().clone()),
                        }
                    }
                    div {
                        label { class: "block text-sm font-medium text-gray-700 dark:text-gray-300", "Region" }
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
                            class: "bg-purple-600 text-white px-4 py-2 rounded hover:bg-purple-700 disabled:opacity-50 disabled:cursor-not-allowed",
                            r#type: "submit",
                            disabled: *is_saving.read(),
                            if *is_saving.read() { "Creating..." } else { "Create Bucket" }
                        }
                    }
                }
            }
        }
    }
}