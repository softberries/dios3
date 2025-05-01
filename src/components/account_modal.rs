use dioxus::prelude::*;
use once_cell::sync::Lazy;
use std::sync::Mutex;
use crate::utils::DB;
use tokio::task::spawn_blocking;
use crate::model::account::Account;

fn save_account_to_db(
    account_id: Option<i64>,
    name: &str,
    description: &str,
    access_key: &str,
    secret_key: &str,
    is_default: &str,
) {
    if let Some(conn) = DB.lock().unwrap().as_ref() {
        if let Some(id) = account_id {
            println!("UPDATING ACCOUNT {}", id);
            conn.execute(
                "UPDATE accounts SET name = ?1, description = ?2, access_key = ?3, secret_key = ?4, is_default = ?5 WHERE id = ?6",
                &[name, description, access_key, secret_key, is_default, &id.to_string()],
            ).expect("Failed to update account");
        } else {
            println!("INSERTING NEW ACCOUNT");
            conn.execute(
                "INSERT INTO accounts (name, description, access_key, secret_key, is_default) VALUES (?1, ?2, ?3, ?4, ?5)",
                &[name, description, access_key, secret_key, is_default],
            ).expect("Failed to insert account");
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct AccountModalProps {
    show_modal: Signal<bool>,
    selected_account: Signal<Option<Account>>,
}

#[component]
pub fn AccountModal(mut props: AccountModalProps) -> Element {
    let account = props.selected_account.read().clone();
    let mut account_name = use_signal(|| account.as_ref().map(|a| a.name.clone()).unwrap_or_default());
    let mut short_description = use_signal(|| account.as_ref().map(|a| a.description.clone()).unwrap_or_default());
    let mut access_key = use_signal(|| account.as_ref().map(|a| a.access_key.clone()).unwrap_or_default());
    let mut secret_key = use_signal(|| account.as_ref().map(|a| a.secret_key.clone()).unwrap_or_default());
    let mut is_default = use_signal(|| account.as_ref().map(|a| a.is_default.clone()).unwrap_or_default());

    rsx! {
        div {
            class: "fixed inset-0 z-50 w-screen h-screen flex items-center justify-center bg-black bg-opacity-50",
            onclick: move |_| props.show_modal.set(false),
            div {
                class: "bg-white dark:bg-gray-800 rounded-lg p-6 shadow-xl w-full max-w-md",
                onclick: move |e| e.stop_propagation(), // prevent click from closing the modal

                h2 { class: "text-xl font-bold mb-4 text-gray-900 dark:text-gray-100", if account.is_some() { "Edit Account" } else { "New Account" } }

                form {
                    class: "space-y-4",
                    onsubmit: move |evt| {
                        evt.prevent_default();

                        let name = account_name.read().clone();
                        let description = short_description.read().clone();
                        let access_key = access_key.read().clone();
                        let secret_key = secret_key.read().clone();
                        let account_id = account.as_ref().map(|a| a.id);

                        spawn_blocking(move || {
                            save_account_to_db(account_id, &name, &description, &access_key, &secret_key, "true");
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
                            onchange: move |e| is_default.set("true".to_owned()),
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