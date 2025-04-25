use std::time::Duration;
use dioxus::prelude::*;

#[component]
pub fn SalesCard() -> Element {

    rsx!(
            div {
        class: "flex items-center p-4 bg-white rounded-lg shadow-xs dark:bg-gray-800",
        div {
            class: "p-3 mr-4 text-blue-500 bg-blue-100 rounded-full dark:text-blue-100 dark:bg-blue-500",
            svg {
                class: "w-5 h-5",
                fill: "currentColor",
                view_box: "0 0 20 20",
                path {
                    d: "M3 1a1 1 0 000 2h1.22l.305 1.222a.997.997 0 00.01.042l1.358 5.43-.893.892C3.74 11.846 4.632 14 6.414 14H15a1 1 0 000-2H6.414l1-1H14a1 1 0 00.894-.553l3-6A1 1 0 0017 3H6.28l-.31-1.243A1 1 0 005 1H3zM16 16.5a1.5 1.5 0 11-3 0 1.5 1.5 0 013 0zM6.5 18a1.5 1.5 0 100-3 1.5 1.5 0 000 3z"
                }
            }
        }
        div {
            p {
                class: "mb-2 text-sm font-medium text-gray-600 dark:text-gray-400",
                "New sales"
            }
            p {
                class: "text-lg font-semibold text-gray-700 dark:text-gray-200",
                "376"
            }
        }
    }
    )
}
