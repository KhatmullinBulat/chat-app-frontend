use dioxus::{
    logger::tracing::{self},
    prelude::*,
};
use dioxus_free_icons::{
    icons::bs_icons::{BsPersonCircle, BsSend},
    Icon,
};
use dioxus_primitives::scroll_area::ScrollDirection;
use futures_util::stream::StreamExt;
use gloo_net::{
    http::Request,
    websocket::{futures::WebSocket, Message},
};
use serde::Deserialize;
use web_sys::window;

use crate::{
    components::{self, button::Button, scroll_area::ScrollArea, textarea::Textarea},
    router::Route,
};

#[derive(Deserialize)]
struct IncomingMsg {
    text: String,
    user: String,
}

async fn send_message(name: String, text: String) -> Result<(), String> {
    let url = "https://gleaming-katrinka-chipolino-f7cae282.koyeb.app/send";

    match Request::post(url)
        .header("Content-Type", "application/json")
        .json(&serde_json::json!({"text": text, "user": name}))
    {
        Ok(req_builder) => match req_builder.send().await {
            Ok(resp) => {
                if resp.ok() {
                    tracing::info!("Сообщение отправлено: статус {}", resp.status());
                    Ok(())
                } else {
                    let err_msg =
                        format!("HTTP ошибка: {} - {}", resp.status(), resp.status_text());
                    tracing::error!("{}", err_msg);
                    Err(err_msg)
                }
            }
            Err(e) => {
                let err_msg = match e {
                    gloo_net::Error::SerdeError(serde_err) => {
                        format!("Serde ошибка: {}", serde_err)
                    }
                    gloo_net::Error::GlooError(gloo_err) => format!("Gloo ошибка: {}", gloo_err),
                    gloo_net::Error::JsError(js_err) => format!("JS ошибка: {:?}", js_err),
                };
                tracing::error!("{}", err_msg);
                Err(err_msg)
            }
        },
        Err(e) => Err(format!("Ошибка запроса: {:?}", e)),
    }
}

#[component]
pub fn Home() -> Element {
    let navigator = use_navigator();
    let mut username = use_signal(|| None::<String>);

    use_effect(move || {
        let name = window()
            .and_then(|w| w.local_storage().ok().flatten())
            .and_then(|s| s.get("username").ok().flatten());

        match name {
            Some(name) => username.set(Some(name)),
            None => {
                let _ = navigator.replace(Route::Login {});
            }
        }
    });

    let Some(_) = username() else {
        return rsx! {};
    };

    let name = username.unwrap();

    let mut current_message = use_signal(|| String::new());
    let mut messages = use_signal(|| Vec::<IncomingMsg>::new());
    let mut err_msg = use_signal(|| None::<String>);

    use_coroutine(move |_: UnboundedReceiver<String>| async move {
        let ws_url = "https://gleaming-katrinka-chipolino-f7cae282.koyeb.app/ws";

        let ws = WebSocket::open(ws_url).expect("Не удалось открыть WS");
        let (_, mut read) = ws.split();

        while let Some(msg) = read.next().await {
            match msg {
                Ok(Message::Text(json_text)) => {
                    match serde_json::from_str::<IncomingMsg>(&json_text) {
                        Ok(msg) => {
                            messages.write().push(msg);
                        }
                        Err(err) => {
                            tracing::warn!("Не удалось распарсить сообщение: {}", err);
                        }
                    }
                }
                _ => {}
            }
        }

        tracing::info!("Websocket закрылся");
    });

    rsx! {
        div { class: "h-dvh flex flex-col justify-end gap-3 max-w-[800px] mx-auto py-10 px-5",

            if let Some(err) = err_msg() {
                div { class: "text-red-500", "{err}" }
            }

            button {
                class: "self-center mb-auto cursor-pointer",
                onclick: move |_| {
                    navigator.push(Route::Login {});
                },
                Icon { icon: BsPersonCircle, height: 35, width: 35 }
            }

            ScrollArea {
                class: "scroll-content",
                direction: ScrollDirection::Vertical,
                always_show_scrollbars: true,
                div { class: "flex flex-col items-center gap-2",
                    for msg in messages.read().iter() {
                        MessageCard {
                            text: msg.text.clone(),
                            user: msg.user.clone(),
                            current_user: name.clone(),
                        }
                    }
                }
            }

            form {
                class: "flex justify-end",
                onsubmit: move |e| {
                    e.prevent_default();

                    let msg = current_message.read().clone();
                    let name_clone = name.clone();

                    if !msg.is_empty() {
                        spawn(async move {
                            match send_message(name_clone, msg).await {
                                Ok(_) => {}
                                Err(err) => err_msg.set(Some(err)),
                            }
                        });

                        current_message.set("".to_string());
                    }
                },

                Textarea {
                    class: "textarea",
                    variant: components::textarea::TextareaVariant::Outline,
                    placeholder: "Как дела?",
                    value: current_message,
                    maxlength: 300,
                    oninput: move |e: FormEvent| {
                        current_message.set(e.value());
                    },
                }

                Button { class: "button", r#type: "submit",
                    Icon { icon: BsSend }
                }
            }
        }
    }
}

#[component]
fn MessageCard(text: String, user: String, current_user: String) -> Element {
    let is_my_message = user == current_user;

    let base = "max-w-[75%] rounded-2xl px-4 py-3 text-sm leading-relaxed break-words shadow-lg";

    let class = if is_my_message {
        format!(
            "{} bg-gradient-to-br from-blue-500 to-blue-600 text-white",
            base
        )
    } else {
        format!("{} bg-gray-200 text-gray-900", base)
    };

    rsx! {
        div { class: "{class}", "{user}: {text}" }
    }
}
