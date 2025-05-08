mod pages;
mod components;
mod services;
mod model;

mod settings;
mod utils;

mod repositories;

use dioxus::prelude::*;
use pages::Buckets;
use pages::Dashboard;
use pages::Accounts;
use components::SettingsModal;
use components::TopBar;
use components::LeftSidebar;
use crate::utils::{init_db, init_state};

#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
enum Route {
    #[layout(MainContent)]
    #[route("/")]
    Dashboard {},
    #[route("/buckets")]
    Buckets {},
    #[route("/accounts")]
    Accounts {},
    // #[route("/blog/:id")]
    // Blog { id: i32 },
}

const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/main.css");
const HEADER_SVG: Asset = asset!("/assets/header.svg");
const LOGO_SMALL: Asset = asset!("/assets/dios3_small.png");
const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");

fn main() {
    // let rt = tokio::runtime::Runtime::new().unwrap();
    init_db();
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    init_state();
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

