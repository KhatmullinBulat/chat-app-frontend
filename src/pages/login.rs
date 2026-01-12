use dioxus::prelude::*;
use dioxus_free_icons::icons::bs_icons::{BsKey, BsPerson};
use dioxus_free_icons::Icon;
use web_sys::window;

use crate::router::Route;

#[component]
pub fn Login() -> Element {
    let mut name = use_signal(|| String::new());
    let storage = window().unwrap().local_storage().unwrap().unwrap();
    let storage_clone = storage.clone();
    storage_clone.set("username", "Гость").unwrap();

    rsx! {
        div { class: "h-dvh flex flex-col items-center justify-center bg-gradient p-4",

            div { class: "max-w-xl w-full rounded-3xl rounded-b-none card-shadow bg-white p-8",
                h1 { class: "text-2xl lg:text-4xl font-bold text-gray-800 mb-3",
                    "С возвращением!"
                }

                p { class: "text-gray-600",
                    "Войдите в свой аккаунт, чтобы продолжить общение"
                }
            }
            div { class: "p-8 text-white card-shadow rounded-3xl rounded-t-none max-w-xl w-full",
                h2 { class: "text-2xl lg:text-3xl font-bold mb-2", "Вход в аккаунт" }
                p { class: "text-blue-100 mb-8",
                    "Пожалуйста, введите свои учетные данные"
                }

                form {
                    class: "flex flex-col gap-8",
                    onsubmit: move |e: FormEvent| {
                        e.prevent_default();

                        let nav = navigator();

                        if !name.is_empty() {
                            storage_clone.set("username", &name.read()).unwrap();
                        }

                        nav.push(Route::Home {});
                    },
                    div {
                        div { class: "flex gap-2",
                            Icon { icon: BsPerson }
                            label {
                                r#for: "name",
                                class: "block text-sm font-medium mb-2",
                                "Имя"
                            }
                        }
                        input {
                            id: "name",
                            name: "name",
                            required: true,
                            class: "w-full pl-3 pr-4 py-3 backdrop-blur-sm bg-white/10 border border-white/20 rounded-xl text-white placeholder-blue-200 focus:outline-none input-focus mb-2",
                            placeholder: "Ваше имя",
                            oninput: move |e: FormEvent| {
                                name.set(e.value());
                            },
                        }
                        p { class: "text-blue-200 text-xs",
                            "Введите ваше имя или никнейм"
                        }
                    }

                    div {
                        div { class: "flex gap-2",
                            Icon { width: 22, icon: BsKey }
                            label {
                                r#for: "password",
                                class: "block text-sm font-medium mb-2",
                                "Пароль"
                            }
                        }

                        input {
                            id: "password",
                            name: "password",
                            required: false,
                            class: "w-full pl-3 pr-4 py-3 backdrop-blur-sm bg-white/10 border border-white/20 rounded-xl text-white placeholder-blue-200 focus:outline-none input-focus mb-2",
                            placeholder: "Пароль",
                        }
                        p { class: "text-blue-200 text-xs",
                            "Внимание пароль в данный момент необязателен"
                        }
                    }

                    button {
                        r#type: "submit",
                        class: "w-full cursor-pointer bg-white text-blue-600 font-semibold py-3 px-4 rounded-xl hover:bg-blue-50 btn-transition focus:outline-none focus:ring-4 focus:ring-white/30",
                        "Войти в систему"
                    }
                }
            }
        }
    }
}
