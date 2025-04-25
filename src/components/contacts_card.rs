use std::time::Duration;
use dioxus::prelude::*;

#[component]
pub fn ContactsCard() -> Element {
    rsx!(
           div {
        class: "flex items-center p-4 bg-white rounded-lg shadow-xs dark:bg-gray-800",
        div {
            class: "p-3 mr-4 text-teal-500 bg-teal-100 rounded-full dark:text-teal-100 dark:bg-teal-500",
            svg {
                class: "w-5 h-5",
                fill: "currentColor",
                view_box: "0 0 20 20",
                path {
                    fill_rule: "evenodd",
                    clip_rule: "evenodd",
                    d: "M18 5v8a2 2 0 01-2 2h-5l-5 4v-4H4a2 2 0 01-2-2V5a2 2 0 012-2h12a2 2 0 012 2zM7 8H5v2h2V8zm2 0h2v2H9V8zm6 0h-2v2h2V8z"
                }
            }
        }
        div {
            p {
                class: "mb-2 text-sm font-medium text-gray-600 dark:text-gray-400",
                "Pending contacts"
            }
            p {
                class: "text-lg font-semibold text-gray-700 dark:text-gray-200",
                "35"
            }
        }
    }
    )
}
