mod pages;
mod components;

use dioxus::prelude::*;
use pages::Buckets;
use pages::Dashboard;
use components::SettingsModal;

#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
enum Route {
    #[layout(MainContent)]
    #[route("/")]
    Dashboard {},
    #[route("/buckets")]
    Buckets {},
    // #[route("/blog/:id")]
    // Blog { id: i32 },
}

const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/main.css");
const HEADER_SVG: Asset = asset!("/assets/header.svg");
const LOGO_SMALL: Asset = asset!("/assets/dios3_small.png");
const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS } document::Link { rel: "stylesheet", href: TAILWIND_CSS }
        Router::<Route> {}
    }
}

#[component]
pub fn MainContent() -> Element {
    rsx!(
    div { class: "flex h-screen bg-gray-50 dark:bg-gray-900",
        // Sidebar
        aside { class: "hidden w-64 bg-white dark:bg-gray-800 md:block",
            div { class: "py-4 text-gray-500 dark:text-gray-400",
                a { class: "ml-6 text-lg font-bold text-gray-800 dark:text-gray-200", href: "#",
                "DioS3"
                }
                LeftSidebar {}
            }
        }
        // Main content wrapper
        div { class: "flex flex-col flex-1 w-full",
            // Top bar
            TopBar {},
            // Main content
            Outlet::<Route> {}
        }
    },

)
}

#[component]
fn TopBar() -> Element {
    let mut show_modal = use_signal(|| false);

    rsx! {
        if *show_modal.read() {
                    SettingsModal {
                        show_modal: show_modal.clone()
                    }
                }
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
                        button {
                            class: "relative align-middle rounded-md focus:outline-none focus:shadow-outline-purple",
                            aria_label: "Notifications",
                            onclick: move |_| show_modal.set(true),
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

                // Profile menu
                li {
                    class: "relative",
                    button {
                        class: "align-middle rounded-full focus:shadow-outline-purple focus:outline-none",
                        aria_label: "Account",
                        aria_haspopup: "true",
                        img {
                            class: "object-cover w-8 h-8 rounded-full",
                            src: "https://images.unsplash.com/photo-1502378735452-bc7d86632805?ixlib=rb-0.3.5&q=80&fm=jpg&crop=entropy&cs=tinysrgb&w=200&fit=max&s=aa3a807e1bbdfd4364d1f449eaa96d82",
                            alt: "",
                            aria_hidden: "true"
                        }
                    }
                }
            }
        }
    }
}
}
#[component]
fn LeftSidebar() -> Element {
    rsx!(
    ul { class: "mt-6",
        li { class: "relative px-6 py-3",
            span {
                class: "absolute inset-y-0 left-0 w-1 bg-purple-600 rounded-tr-lg rounded-br-lg",
                aria_hidden: "true"
            }
            Link {
                to: Route::Dashboard {},
                class: "inline-flex items-center w-full text-sm font-semibold text-gray-800 transition-colors duration-150 hover:text-gray-800 dark:hover:text-gray-200 dark:text-gray-100",
                svg {
                    class: "w-5 h-5",
                    fill: "none",
                    stroke_linecap: "round",
                    stroke_linejoin: "round",
                    stroke_width: "2",
                    view_box: "0 0 24 24",
                    stroke: "currentColor",
                    path {
                        d: "M3 12l2-2m0 0l7-7 7 7M5 10v10a1 1 0 001 1h3m10-11l2 2m-2-2v10a1 1 0 01-1 1h-3m-6 0a1 1 0 001-1v-4a1 1 0 011-1h2a1 1 0 011 1v4a1 1 0 001 1m-6 0h6"
                    }
                }
                span { class: "ml-4", "Dashboard" }
            }
        }
    },
        ul {
        li { class: "relative px-6 py-3",
            Link { to: Route::Buckets {}, class: "inline-flex items-center w-full text-sm font-semibold transition-colors duration-150 hover:text-gray-800 dark:hover:text-gray-200",
                svg {
                    class: "w-5 h-5", fill: "none",
                    stroke_linecap: "round", stroke_linejoin: "round", stroke_width: "2",
                    view_box: "0 0 24 24", stroke: "currentColor",
                    path { d: "M9 5H7a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2m-3 7h3m-3 4h3m-6-4h.01M9 16h.01" }
                }
                span { class: "ml-4", "Buckets" }
            }
        }
        li { class: "relative px-6 py-3",
            a { class: "inline-flex items-center w-full text-sm font-semibold transition-colors duration-150 hover:text-gray-800 dark:hover:text-gray-200", href: "cards.html",
                svg {
                    class: "w-5 h-5", fill: "none",
                    stroke_linecap: "round", stroke_linejoin: "round", stroke_width: "2",
                    view_box: "0 0 24 24", stroke: "currentColor",
                    path { d: "M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10" }
                }
                span { class: "ml-4", "Jobs" }
            }
        }
        li { class: "relative px-6 py-3",
            a { class: "inline-flex items-center w-full text-sm font-semibold transition-colors duration-150 hover:text-gray-800 dark:hover:text-gray-200", href: "charts.html",
                svg {
                    class: "w-5 h-5", fill: "none",
                    stroke_linecap: "round", stroke_linejoin: "round", stroke_width: "2",
                    view_box: "0 0 24 24", stroke: "currentColor",
                    path { d: "M11 3.055A9.001 9.001 0 1020.945 13H11V3.055z" }
                    path { d: "M20.488 9H15V3.512A9.025 9.025 0 0120.488 9z" }
                }
                span { class: "ml-4", "Permissions" }
            }
        }
        li { class: "relative px-6 py-3",
            a { class: "inline-flex items-center w-full text-sm font-semibold transition-colors duration-150 hover:text-gray-800 dark:hover:text-gray-200", href: "buttons.html",
                svg {
                    class: "w-5 h-5", fill: "none",
                    stroke_linecap: "round", stroke_linejoin: "round", stroke_width: "2",
                    view_box: "0 0 24 24", stroke: "currentColor",
                    path { d: "M15 15l-2 5L9 9l11 4-5 2zm0 0l5 5M7.188 2.239l.777 2.897M5.136 7.965l-2.898-.777M13.95 4.05l-2.122 2.122m-5.657 5.656l-2.12 2.122" }
                }
                span { class: "ml-4", "Statistics" }
            }
        }
        li { class: "relative px-6 py-3",
            a { class: "inline-flex items-center w-full text-sm font-semibold transition-colors duration-150 hover:text-gray-800 dark:hover:text-gray-200", href: "modals.html",
                svg {
                    class: "w-5 h-5", fill: "none",
                    stroke_linecap: "round", stroke_linejoin: "round", stroke_width: "2",
                    view_box: "0 0 24 24", stroke: "currentColor",
                    path { d: "M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z" }
                }
                span { class: "ml-4", "Modals" }
            }
        }
        li { class: "relative px-6 py-3",
            a { class: "inline-flex items-center w-full text-sm font-semibold transition-colors duration-150 hover:text-gray-800 dark:hover:text-gray-200", href: "tables.html",
                svg {
                    class: "w-5 h-5", fill: "none",
                    stroke_linecap: "round", stroke_linejoin: "round", stroke_width: "2",
                    view_box: "0 0 24 24", stroke: "currentColor",
                    path { d: "M4 6h16M4 10h16M4 14h16M4 18h16" }
                }
                span { class: "ml-4", "Pro Version" }
            }
        }
    }
)
}


