use std::time::Duration;
use dioxus::prelude::*;
use dioxus::hooks::{use_coroutine, use_signal};
use crate::components::{AccountCard, AccountModal, ClientsCard, ContactsCard, GithubStarAction, SalesCard};
use tokio::task::spawn_blocking;
use crate::model::account::Account;
use crate::repositories::account_repo;
use crate::repositories::account_repo::{delete_account, fetch_accounts, fetch_accounts_paginated};
use crate::utils::CURRENT_ACCOUNT;

const S3_IMG: Asset = asset!("/assets/aws_logo.png");

async fn list_accounts(page: Option<usize>, page_size: Option<usize>) -> (Vec<Account>, usize) {
    let result = spawn_blocking(move || fetch_accounts_paginated(page, page_size)).await;
    match result {
        Ok((accounts, total)) => (accounts, total),
        Err(e) => {
            println!("Failed to fetch accounts: {:?}", e);
            (Vec::new(), 0)
        }
    }
}

/// Home page
#[component]
pub fn Accounts() -> Element {
    let mut show_modal = use_signal(|| false);
    let accounts = use_signal(|| Vec::<Account>::new());
    let selected_account = use_signal(|| None as Option<Account>);
    let mut refresh_accounts = use_signal(|| false);
    let mut account_to_delete = use_signal(|| None as Option<Account>);
    let mut current_page = use_signal(|| 0usize);
    let mut page_size = use_signal(|| 20usize);
    let mut total_accounts = use_signal(|| 0usize);

    use_effect(move || {
        let mut accounts_signal = accounts.clone();
        let page = current_page.read().clone();
        let size = page_size.read().clone();
        spawn(async move {
            let (account_data, total) = list_accounts(Some(page), Some(size)).await;
            accounts_signal.set(account_data);
            total_accounts.set(total);
        });
    });

    use_effect(move || {
        if *refresh_accounts.read() {
            let mut accounts_signal = accounts.clone();
            let page = current_page.read().clone();
            let size = page_size.read().clone();
            spawn(async move {
                let (account_data, total) = list_accounts(Some(page), Some(size)).await;
                accounts_signal.set(account_data);
                total_accounts.set(total);
                refresh_accounts.set(false);
            });
        }
    });

    // Effect to reload accounts when page or page_size changes
    use_effect(move || {
        let mut accounts_signal = accounts.clone();
        let page = current_page.read().clone();
        let size = page_size.read().clone();
        spawn(async move {
            let (account_data, total) = list_accounts(Some(page), Some(size)).await;
            accounts_signal.set(account_data);
            total_accounts.set(total);
        });
    });

    rsx!(
        if *show_modal.read() {
                    AccountModal {
                        show_modal: show_modal.clone(),
                        selected_account: selected_account.clone(),
                        refresh_accounts: refresh_accounts.clone(),
                    }
                },
        if let Some(acc) = account_to_delete.read().as_ref() {
                div { class: "fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center",
                    div { class: "bg-white dark:bg-gray-800 p-6 rounded shadow",
                        h2 { class: "text-lg font-bold mb-4", "Confirm Delete" }
                        p { "Are you sure you want to delete account: ", {acc.name.clone()}, "?" }
                        div { class: "flex justify-end space-x-2",
                            button {
                                class: "px-4 py-2 bg-gray-300 rounded hover:bg-gray-400",
                                onclick: move |_| account_to_delete.set(None),
                                "Cancel"
                            }
                            button {
                                class: "px-4 py-2 bg-red-600 text-white rounded hover:bg-red-700",
                                onclick: {
                                    let account_id = acc.id;
                                    let mut account_to_delete = account_to_delete.clone();
                                    let mut refresh_accounts = refresh_accounts.clone();
                                    move |_| {
                                        spawn_blocking(move || {
                                            delete_account(account_id);
                                        });
                                        account_to_delete.set(None);
                                        refresh_accounts.set(true);
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
                            "Accounts"
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
                                "New Account"
                            }
                        }
                    }
                    GithubStarAction {},
                    AccountsTable { 
                        accounts: accounts.read().clone(), 
                        selected_account: selected_account.clone(), 
                        show_modal: show_modal.clone(), 
                        account_to_delete: account_to_delete.clone(),
                        current_page: current_page.clone(),
                        page_size: page_size.clone(),
                        total_accounts: total_accounts.clone(),
                    }
                }
            }
    )
}

#[component]
fn AccountsTable(accounts: Vec<Account>, selected_account: Signal<Option<Account>>, show_modal: Signal<bool>, account_to_delete: Signal<Option<Account>>, current_page: Signal<usize>, page_size: Signal<usize>, total_accounts: Signal<usize>) -> Element {
    rsx! {
    div { class: "w-full overflow-hidden rounded-lg shadow-xs",
        div { class: "w-full overflow-x-auto",
            table { class: "w-full whitespace-no-wrap",
                thead {
                    tr {
                        class: "text-xs font-semibold tracking-wide text-left text-gray-500 uppercase border-b dark:border-gray-700 bg-gray-50 dark:text-gray-400 dark:bg-gray-800",
                        th { class: "px-4 py-3", "Name" }
                        th { class: "px-4 py-3", "Access Key" }
                        th { class: "px-4 py-3", "Secret Key" }
                        th { class: "px-4 py-3", "Region" }
                        th { class: "px-4 py-3", "Last Accessed" }
                        th { class: "px-4 py-3", "Default?" }
                        th { class: "px-4 py-3", "Actions" }
                    }
                }
                tbody { class: "bg-white divide-y dark:divide-gray-700 dark:bg-gray-800",
                    {accounts.into_iter().map(|acc| {
                        let acc_for_edit = acc.clone();
                        let acc_for_delete = acc.clone();
                        let mut selected_account = selected_account.clone();
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
                                            p { class: "font-semibold", "{acc.name}" }
                                            p { class: "text-xs text-gray-600 dark:text-gray-400", "{acc.description}" }
                                        }
                                    }
                                }
                            td { class: "px-4 py-3 text-sm", "{acc.access_key}" }
                            td { class: "px-4 py-3 text-xs",
                                span {
                                    class: "px-2 py-1 font-semibold leading-tight text-green-700 bg-green-100 rounded-full dark:bg-green-700 dark:text-green-100",
                                    "{acc.masked_secret_key()}"
                                }
                            }
                            td { class: "px-4 py-3 text-sm", "{acc.default_region}" }
                            td { class: "px-4 py-3 text-sm", "-" }
                            td { class: "px-4 py-3 text-sm",
                                input {
                                    r#type: "checkbox",
                                    checked: acc.is_default,
                                    class: "form-checkbox h-5 w-5 text-purple-600 pointer-events-none focus:outline-none",
                                    tabindex: "-1"
                                }
                            }
                            td { class: "px-4 py-3 space-x-2",
                        button {
                            class: "px-2 py-1 text-sm text-white bg-blue-500 rounded hover:bg-blue-600 focus:outline-none",
                            onclick: {
                                let mut selected_account = selected_account.clone();
                                let mut show_modal = show_modal.clone();
                                move |_| {
                                    selected_account.set(Some(acc_for_edit.clone()));
                                    show_modal.set(true);
                                }
                            },
                            "Edit"
                        }
                                button {
                                    class: "px-2 py-1 text-sm text-white bg-red-500 rounded hover:bg-red-600 focus:outline-none",
                                    onclick: {
                                        let mut account_to_delete = account_to_delete.clone();
                                        move |_| {
                                            account_to_delete.set(Some(acc_for_delete.clone()));
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
                    let total = *total_accounts.read();
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
                        {
                            // Calculate page numbers to display
                            let current = *current_page.read();
                            let size = *page_size.read();
                            let total = *total_accounts.read();
                            let total_pages = if total > 0 { ((total as f64) / (size as f64)).ceil() as usize } else { 1 };
                            
                            // Calculate start and end pages (max 7 pages)
                            let max_visible = 7;
                            let half_visible = max_visible / 2; // 3
                            
                            let (start_page, end_page) = if total_pages <= max_visible {
                                (0, total_pages)
                            } else if current < half_visible {
                                (0, max_visible)
                            } else if current >= total_pages - half_visible {
                                (total_pages - max_visible, total_pages)
                            } else {
                                (current - half_visible, current + half_visible + 1)
                            };
                            
                            let current_page_signal = current_page.clone();
                            
                            (start_page..end_page).map(move |page| {
                                let is_current = page == current;
                                let mut page_signal = current_page_signal.clone();
                                rsx!(
                                    li {
                                        button {
                                            class: if is_current {
                                                "px-3 py-1 text-white transition-colors duration-150 bg-purple-600 border border-r-0 border-purple-600 rounded-md focus:outline-none focus:shadow-outline-purple"
                                            } else {
                                                "px-3 py-1 rounded-md focus:outline-none focus:shadow-outline-purple hover:bg-purple-100 dark:hover:bg-purple-900"
                                            },
                                            onclick: move |_| page_signal.set(page),
                                            "{page + 1}"
                                        }
                                    }
                                )
                            })
                        }
                        li {
                            button {
                                class: "px-3 py-1 rounded-md rounded-r-lg focus:outline-none focus:shadow-outline-purple disabled:opacity-50 disabled:cursor-not-allowed",
                                aria_label: "Next",
                                disabled: {
                                    let current = *current_page.read();
                                    let size = *page_size.read();
                                    let total = *total_accounts.read();
                                    let max_page = if total > 0 { (total - 1) / size } else { 0 };
                                    current >= max_page
                                },
                                onclick: move |_| {
                                    let current = *current_page.read();
                                    let size = *page_size.read();
                                    let total = *total_accounts.read();
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