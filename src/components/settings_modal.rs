use dioxus::prelude::*;
#[derive(Props, Clone, PartialEq)]
pub struct SettingsModalProps {
    show_modal: Signal<bool>,
}

#[component]
pub fn SettingsModal(mut props: SettingsModalProps) -> Element {
    rsx! {
        div {
            class: "fixed inset-0 z-50 w-screen h-screen flex items-center justify-center bg-black bg-opacity-50",
            onclick: move |_| props.show_modal.set(false),
            div {
                class: "bg-white dark:bg-gray-800 rounded-lg p-6 shadow-xl w-full max-w-md",
                onclick: move |e| e.stop_propagation(), // prevent click from closing the modal

                h2 { class: "text-xl font-bold mb-4 text-gray-900 dark:text-gray-100", "Settings" }

                form {
                    class: "space-y-4",
                    onsubmit: move |evt| {
                        evt.prevent_default();
                        props.show_modal.set(false); // optionally close the modal after save
                    },
                    div {
                        label { class: "block text-sm font-medium text-gray-700 dark:text-gray-300", "Username" }
                        input {
                            class: "w-full px-3 py-2 border rounded-md dark:bg-gray-700 dark:text-white",
                            r#type: "text",
                            placeholder: "Enter your name"
                        }
                    }
                    div {
                        label { class: "block text-sm font-medium text-gray-700 dark:text-gray-300", "Email" }
                        input {
                            class: "w-full px-3 py-2 border rounded-md dark:bg-gray-700 dark:text-white",
                            r#type: "email",
                            placeholder: "example@mail.com"
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