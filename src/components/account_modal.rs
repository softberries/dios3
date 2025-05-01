use dioxus::prelude::*;
use once_cell::sync::Lazy;
use std::sync::Mutex;
use crate::utils::DB;
use tokio::task::spawn_blocking;

fn save_account_to_db(
    name: &str,
    description: &str,
    access_key: &str,
    secret_key: &str,
    is_default: bool,
) {
    if let Some(conn) = DB.lock().unwrap().as_ref() {
        println!("SAVING ACCOUNT");
        conn.execute(
            "INSERT INTO accounts (name, description, access_key, secret_key, is_default) VALUES (?1, ?2, ?3, ?4, ?5)",
            &[name, description, access_key, secret_key, "true"],
        ).expect("Failed to insert account");
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct AccountModalProps {
    show_modal: Signal<bool>,
}

#[component]
pub fn AccountModal(mut props: AccountModalProps) -> Element {
    let mut account_name = use_signal(String::new);
    let mut short_description = use_signal(String::new);
    let mut access_key = use_signal(String::new);
    let mut secret_key = use_signal(String::new);
    let mut set_default = use_signal(String::new);

    rsx! {
        div {
            class: "fixed inset-0 z-50 w-screen h-screen flex items-center justify-center bg-black bg-opacity-50",
            onclick: move |_| props.show_modal.set(false),
            div {
                class: "bg-white dark:bg-gray-800 rounded-lg p-6 shadow-xl w-full max-w-md",
                onclick: move |e| e.stop_propagation(), // prevent click from closing the modal

                h2 { class: "text-xl font-bold mb-4 text-gray-900 dark:text-gray-100", "New Account" }

                form {
                    class: "space-y-4",
                    onsubmit: move |evt| {
                        evt.prevent_default();

                        let name = account_name.read().clone();
                        let description = short_description.read().clone();
                        let access_key = access_key.read().clone();
                        let secret_key = secret_key.read().clone();

                        spawn_blocking(move || {
                            save_account_to_db(&name, &description, &access_key, &secret_key, true);
                        });

                        props.show_modal.set(false);
                    },
                    div {
                        label { class: "block text-sm font-medium text-gray-700 dark:text-gray-300", "Account Name" }
                        input {
                            class: "w-full px-3 py-2 border rounded-md dark:bg-gray-700 dark:text-white",
                            r#type: "text",
                            placeholder: "Enter account name",
                            value: "{account_name}",
                            oninput: move |e| account_name.set(e.value().clone()),
                        }
                    }
                     div {
                        label { class: "block text-sm font-medium text-gray-700 dark:text-gray-300", "Short Description" }
                        input {
                            class: "w-full px-3 py-2 border rounded-md dark:bg-gray-700 dark:text-white",
                            r#type: "text",
                            placeholder: "Enter short description (eg. prod/staging/dev)",
                            value: "{short_description}",
                            oninput: move |e| short_description.set(e.value().clone()),
                        }
                    }
                    div {
                        label { class: "block text-sm font-medium text-gray-700 dark:text-gray-300", "Access Key" }
                        input {
                            class: "w-full px-3 py-2 border rounded-md dark:bg-gray-700 dark:text-white",
                            r#type: "text",
                            placeholder: "ACCESS_KEY",
                            value: "{access_key}",
                            oninput: move |e| access_key.set(e.value().clone()),
                        }
                    }
                    div {
                        label { class: "block text-sm font-medium text-gray-700 dark:text-gray-300", "Secret Key" }
                        input {
                            class: "w-full px-3 py-2 border rounded-md dark:bg-gray-700 dark:text-white",
                            r#type: "password",
                            placeholder: "SECRET_KEY",
                            value: "{secret_key}",
                            oninput: move |e| secret_key.set(e.value().clone()),
                        }
                    }
                    div {
                        label { class: "block text-sm font-medium text-gray-700 dark:text-gray-300", "Set Default" }
                        input {
                            class: "w-full px-3 py-2 border rounded-md dark:bg-gray-700 dark:text-white",
                            r#type: "checkbox",
                            // checked: "{set_default}",
                            checked: "true",
                            onchange: move |e| set_default.set("true".to_owned()),
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