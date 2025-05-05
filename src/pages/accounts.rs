use std::arch::aarch64::uint32x2_t;
use std::time::Duration;
use dioxus::prelude::*;
use dioxus::hooks::{use_coroutine, use_signal};
use crate::components::{AccountCard, AccountModal, ClientsCard, ContactsCard, SalesCard};
use tokio::task::spawn_blocking;
use crate::model::account::Account;
use crate::repositories::account_repo;
use crate::repositories::account_repo::{delete_account, fetch_accounts};

const S3_IMG: Asset = asset!("/assets/aws_logo.png");
/// Home page
#[component]
pub fn Accounts() -> Element {
    let mut show_modal = use_signal(|| false);
    let accounts = use_signal(|| Vec::<Account>::new());
    let selected_account = use_signal(|| None as Option<Account>);
    let mut refresh_accounts = use_signal(|| false);
    let mut account_to_delete = use_signal(|| None as Option<Account>);

    use_effect(move || {
        let mut accounts = accounts.clone();
        spawn(async move {
            let data = spawn_blocking(move || fetch_accounts());
            accounts.set(data.await.unwrap());
        });
    });

    use_effect(move || {
        if *refresh_accounts.read() {
            let mut accounts = accounts.clone();
            spawn(async move {
                let data = tokio::task::spawn_blocking(move || fetch_accounts()).await.unwrap();
                accounts.set(data);
                refresh_accounts.set(false);
            });
        }
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
                        button {
                            class: "px-4 py-2 mr-4 text-sm font-medium text-white bg-purple-600 rounded-lg hover:bg-purple-700 focus:outline-none focus:ring",
                            onclick: move |_| show_modal.set(true),
                            "New Account"
                        }
                    }
                    GithubStarAction {},
                    AccountsTable { accounts: accounts.read().clone(), selected_account: selected_account.clone(), show_modal: show_modal.clone(), account_to_delete: account_to_delete.clone()}
                }
            }
    )
}

#[component]
fn GithubStarAction() -> Element {
    rsx!(
        a {
    class: "flex items-center justify-between p-4 mb-8 text-sm font-semibold text-purple-100 bg-purple-600 rounded-lg shadow-md focus:outline-none focus:shadow-outline-purple",
    href: "https://github.com/estevanmaito/windmill-dashboard",
    target: "_blank", // optional, for opening in new tab
    rel: "noopener noreferrer", // optional, for security

    div {
        class: "flex items-center",
        svg {
            class: "w-5 h-5 mr-2",
            fill: "currentColor",
            view_box: "0 0 20 20",
            path {
                d: "M9.049 2.927c.3-.921 1.603-.921 1.902 0l1.07 3.292a1 1 0 00.95.69h3.462c.969 0 1.371 1.24.588 1.81l-2.8 2.034a1 1 0 00-.364 1.118l1.07 3.292c.3.921-.755 1.688-1.54 1.118l-2.8-2.034a1 1 0 00-1.175 0l-2.8 2.034c-.784.57-1.838-.197-1.539-1.118l1.07-3.292a1 1 0 00-.364-1.118L2.98 8.72c-.783-.57-.38-1.81.588-1.81h3.461a1 1 0 00.951-.69l1.07-3.292z"
            }
        }
        span { "Star this project on GitHub" }
    }
    span { "View more →" }
}
    )
}

#[component]
fn AccountsTable(accounts: Vec<Account>, selected_account: Signal<Option<Account>>, show_modal: Signal<bool>, account_to_delete: Signal<Option<Account>>) -> Element {
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
                                    "{acc.secret_key}"
                                }
                            }
                            td { class: "px-4 py-3 text-sm", "{acc.default_region}" }
                            td { class: "px-4 py-3 text-sm", "-" }
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