use dioxus::prelude::*;
use crate::components::SettingsModal;
use crate::model::account::Account;
use crate::repositories::account_repo::fetch_accounts;
use crate::utils::CURRENT_ACCOUNT;

#[component]
pub fn TopBar() -> Element {
    let accounts = fetch_accounts();
    rsx! {
    header {
        class: "z-10 py-4 bg-white shadow-md dark:bg-gray-800",
        div {
            class: "container flex items-center justify-between h-full px-6 mx-auto text-purple-600 dark:text-purple-300",

            // Mobile hamburger
            button {
                class: "p-1 mr-5 -ml-1 rounded-md md:hidden focus:outline-none focus:shadow-outline-purple",
                aria_label: "Menu",
                svg {
                    class: "w-6 h-6",
                    fill: "currentColor",
                    view_box: "0 0 20 20",
                    path {
                        fill_rule: "evenodd",
                        clip_rule: "evenodd",
                        d: "M3 5a1 1 0 011-1h12a1 1 0 110 2H4a1 1 0 01-1-1zM3 10a1 1 0 011-1h12a1 1 0 110 2H4a1 1 0 01-1-1zM3 15a1 1 0 011-1h12a1 1 0 110 2H4a1 1 0 01-1-1z"
                    }
                }
            }

            // Search input
            div {
                class: "flex justify-center flex-1 lg:mr-32",
                div {
                    class: "relative w-full max-w-xl mr-6 focus-within:text-purple-500",
                    div {
                        class: "absolute inset-y-0 flex items-center pl-2",
                        svg {
                            class: "w-4 h-4",
                            fill: "currentColor",
                            view_box: "0 0 20 20",
                            path {
                                fill_rule: "evenodd",
                                clip_rule: "evenodd",
                                d: "M8 4a4 4 0 100 8 4 4 0 000-8zM2 8a6 6 0 1110.89 3.476l4.817 4.817a1 1 0 01-1.414 1.414l-4.816-4.816A6 6 0 012 8z"
                            }
                        }
                    }
                    input {
                        class: "w-full pl-8 pr-2 text-sm text-gray-700 placeholder-gray-600 bg-gray-100 border-0 rounded-md dark:placeholder-gray-500 dark:focus:shadow-outline-gray dark:focus:placeholder-gray-600 dark:bg-gray-700 dark:text-gray-200 focus:placeholder-gray-500 focus:bg-white focus:border-purple-300 focus:outline-none focus:shadow-outline-purple form-input",
                        r#type: "text",
                        placeholder: "Search for projects",
                        aria_label: "Search"
                    }
                }
            }

            // Right-side icons
            ul {
                class: "flex items-center flex-shrink-0 space-x-6",

                // Theme toggle
                li {
                    class: "flex",
                    button {
                        class: "rounded-md focus:outline-none focus:shadow-outline-purple",
                        aria_label: "Toggle color mode",
                        svg {
                            class: "w-5 h-5",
                            fill: "currentColor",
                            view_box: "0 0 20 20",
                            path {
                                d: "M10 2a1 1 0 011 1v1a1 1 0 11-2 0V3a1 1 0 011-1zm4 8a4 4 0 11-8 0 4 4 0 018 0zm-.464 4.95l.707.707a1 1 0 001.414-1.414l-.707-.707a1 1 0 00-1.414 1.414zm2.12-10.607a1 1 0 010 1.414l-.706.707a1 1 0 11-1.414-1.414l.707-.707a1 1 0 011.414 0zM17 11a1 1 0 100-2h-1a1 1 0 100 2h1zm-7 4a1 1 0 011 1v1a1 1 0 11-2 0v-1a1 1 0 011-1zM5.05 6.464A1 1 0 106.465 5.05l-.708-.707a1 1 0 00-1.414 1.414l.707.707zm1.414 8.486l-.707.707a1 1 0 01-1.414-1.414l.707-.707a1 1 0 011.414 1.414zM4 11a1 1 0 100-2H3a1 1 0 000 2h1z",
                                fill_rule: "evenodd",
                                clip_rule: "evenodd"
                            }
                        }
                    }
                }

                // Notifications
                li {
                    class: "relative",
                    button {
                        class: "relative align-middle rounded-md focus:outline-none focus:shadow-outline-purple",
                        aria_label: "Notifications",
                        aria_haspopup: "true",
                        svg {
                            class: "w-5 h-5",
                            fill: "currentColor",
                            view_box: "0 0 20 20",
                            path {
                                d: "M10 2a6 6 0 00-6 6v3.586l-.707.707A1 1 0 004 14h12a1 1 0 00.707-1.707L16 11.586V8a6 6 0 00-6-6zM10 18a3 3 0 01-3-3h6a3 3 0 01-3 3z"
                            }
                        }
                        span {
                            aria_hidden: "true",
                            class: "absolute top-0 right-0 inline-block w-3 h-3 transform translate-x-1 -translate-y-1 bg-red-600 border-2 border-white rounded-full dark:border-gray-800"
                        }
                    }
                }
                li {
                    class: "relative",
                    select {
                        class: "px-2 py-1 text-sm bg-white border border-gray-300 rounded-md dark:bg-gray-700 dark:text-white dark:border-gray-600",
                        onchange: move |e| {
                            let selected_name = e.value().clone();
                            if let Some(account) = accounts.iter().find(|a| a.name == selected_name) {
                                CURRENT_ACCOUNT.write().insert(account.clone());
                            }
                        },
                        {accounts.iter().map(|account| {
                            rsx!(
                                option {
                                    value: "{account.name}",
                                    selected: "{account.is_default}",
                                    "{account.name}"
                                }
                            )
                        }).collect::<Vec<_>>().into_iter()}
                    }
                }
            }
        }
    }
}
}