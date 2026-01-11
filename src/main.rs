use crate::components::{button::Button, input::Input, textarea::Textarea};
use dioxus::{
    logger::tracing::{self},
    prelude::*,
};
use futures_util::stream::StreamExt;
use gloo_net::{
    http::Request,
    websocket::{futures::WebSocket, Message},
};
use serde::Deserialize;

mod components;

#[derive(Deserialize)]
struct IncomingMsg {
    text: String,
    user: String,
}

const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/main.css");
const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");
const COMPONENTS: Asset = asset!("/assets/dx-components-theme.css");

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
        Err(e) => {
            // Ошибка на этапе билда запроса (редко)
            Err(format!("Ошибка запроса: {:?}", e))
        }
    }
}

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    let mut name = use_signal(|| String::new());
    let mut current_message = use_signal(|| String::new());
    let mut messages = use_signal(|| Vec::<String>::new());
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
                            let display_text = format!("{}: {}", msg.user, msg.text);
                            messages.write().push(display_text);
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
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        document::Link { rel: "stylesheet", href: TAILWIND_CSS }
        document::Link { rel: "stylesheet", href: COMPONENTS }

        div { class: "flex flex-col gap-3 max-w-[800px] mx-auto py-10 px-5",

            form {
                class: "flex flex-col justify-end gap-1.5",
                onsubmit: move |e| {
                    e.prevent_default();

                    let msg = current_message.read().clone();

                    if !msg.is_empty() {
                        spawn(async move {
                            match send_message(name.read().clone(), msg).await {
                                Ok(_) => {}
                                Err(err) => err_msg.set(Some(err)),
                            }
                        });

                        current_message.set("".to_string());
                    }
                },

                Input {
                    class: "input",
                    placeholder: "Ваше имя",
                    value: name,
                    maxlength: 30,
                    oninput: move |e: FormEvent| {
                        name.set(e.value());
                    },
                }

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

                Button { class: "button self-end", r#type: "submit", "Отправить" }
            }

            if let Some(err) = err_msg() {
                div { class: "text-red-500", "{err}" }
            }

            div {
                class: "overflow-y-auto flex flex-col items-center gap-2",
                id: "messages",

                for msg in messages.read().iter().rev() {
                    MessageCard { text: msg }
                }
            }
        }
    }
}

#[component]
fn MessageCard(text: String) -> Element {
    rsx! {
        div { class: "max-w-[75%] rounded-2xl px-4 py-3
                bg-gradient-to-br from-blue-500 to-blue-600
                text-white shadow-lg text-sm leading-relaxed break-words",
            {text}
        }
    }
}
