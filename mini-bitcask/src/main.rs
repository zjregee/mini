mod ds;
mod kv;
mod utils;
mod config;
mod storage;

use std::env;
use std::net::SocketAddr;
use serde_json::{Value, json};
use tokio::sync::mpsc;
use tokio::sync::oneshot;
use axum::{routing::post, Router, Json, Extension};

enum Operation {
    Get,
    Set,
    SetWithExpire,
    Delete,
    Clear,
    Close,
}

impl Default for Operation {
    fn default() -> Self {
        Operation::Get
    }
}

#[derive(Default)]
struct Message {
    method: Operation,
    key: Option<String>,
    value: Option<String>,
    deadline: Option<u64>,
    channel: Option<oneshot::Sender<(bool, String)>>,
}

async fn kv_get (
    Json(payload): Json<serde_json::Value>,
    Extension(state): Extension<mpsc::Sender<Message>>,
) -> Json<Value>  {
    let key = payload.as_object().unwrap().get("key").unwrap().as_str().unwrap().to_string();
    let (tx, rx) = oneshot::channel();
    let mut message = Message::default();
    message.method = Operation::Get;
    message.key = Some(key);
    message.channel = Some(tx);
    state.send(message).await.unwrap();
    let res = rx.await.unwrap();
    Json(json!({ "status": res.0, "data": res.1 }))
}

async fn kv_set (
    Json(payload): Json<serde_json::Value>,
    Extension(state): Extension<mpsc::Sender<Message>>,
) -> Json<Value> {
    let key = payload.as_object().unwrap().get("key").unwrap().as_str().unwrap().to_string();
    let value = payload.as_object().unwrap().get("value").unwrap().as_str().unwrap().to_string();
    let (tx, rx) = oneshot::channel();
    let mut message = Message::default();
    message.method = Operation::Set;
    message.key = Some(key);
    message.value = Some(value);
    message.channel = Some(tx);
    state.send(message).await.unwrap();
    let res = rx.await.unwrap();
    Json(json!({ "status": res.0 }))
}

async fn kv_set_with_expire (
    Json(payload): Json<serde_json::Value>,
    Extension(state): Extension<mpsc::Sender<Message>>,
) -> Json<Value> {
    let key = payload.as_object().unwrap().get("key").unwrap().as_str().unwrap().to_string();
    let value = payload.as_object().unwrap().get("value").unwrap().as_str().unwrap().to_string();
    let deadline = payload.as_object().unwrap().get("value").unwrap().as_u64().unwrap();
    let (tx, rx) = oneshot::channel();
    let mut message = Message::default();
    message.method = Operation::SetWithExpire;
    message.key = Some(key);
    message.value = Some(value);
    message.deadline = Some(deadline);
    message.channel = Some(tx);
    state.send(message).await.unwrap();
    let res = rx.await.unwrap();
    Json(json!({ "status": res.0 }))
}


async fn kv_delete (
    Json(payload): Json<serde_json::Value>,
    Extension(state): Extension<mpsc::Sender<Message>>,
) -> Json<Value> {
    let key = payload.as_object().unwrap().get("key").unwrap().as_str().unwrap().to_string();
    let (tx, rx) = oneshot::channel();
    let mut message = Message::default();
    message.method = Operation::Delete;
    message.key = Some(key);
    message.channel = Some(tx);
    state.send(message).await.unwrap();
    let res = rx.await.unwrap();
    Json(json!({ "status": res.0 }))
}

async fn kv_clear (
    Extension(state): Extension<mpsc::Sender<Message>>,
) -> Json<Value> {
    let (tx, rx) = oneshot::channel();
    let mut message = Message::default();
    message.method = Operation::Clear;
    message.channel = Some(tx);
    state.send(message).await.unwrap();
    let res = rx.await.unwrap();
    Json(json!({ "status": res.0 }))
}

async fn kv_close (
    Extension(state): Extension<mpsc::Sender<Message>>,
) -> Json<Value> {
    let (tx, rx) = oneshot::channel();
    let mut message = Message::default();
    message.method = Operation::Close;
    message.channel = Some(tx);
    state.send(message).await.unwrap();
    let res = rx.await.unwrap();
    Json(json!({ "status": res.0 }))
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    let port = if args.len() > 1 && args[1].parse::<u16>().is_ok() {
        args[1].parse::<u16>().unwrap()
    } else {
        3010
    };
    let (tx, mut rx) = mpsc::channel(32);
    tx.send(Message::default()).await.unwrap();
    rx.recv().await.unwrap();
    tokio::spawn(async move {
        let app = Router::new()
            .route("/key/get", post(kv_get))
            .route("/key/set", post(kv_set))
            .route("/key/set_with_expire", post(kv_set_with_expire))
            .route("/key/delete", post(kv_delete))
            .route("/key/clear", post(kv_clear))
            .route("/close", post(kv_close))
            .layer(Extension(tx));
        let addr = SocketAddr::from(([127, 0, 0, 1], port));
        println!("listening on {}", addr);
        axum::Server::bind(&addr)
            .serve(app.into_make_service())
            .await
            .unwrap();
    });

    let config = config::default_config();
    let mut db = kv::kv::open(config).expect("kv open internal error");
    while let Some(message) = rx.recv().await {
        match message.method {
            Operation::Get => {
                if let Some(value) = db.get(message.key.unwrap()) {
                    message.channel.unwrap().send((true, value)).unwrap();
                } else {
                    message.channel.unwrap().send((false, "".to_string())).unwrap();
                }
            }
            Operation::Set => {
                if db.set(message.key.unwrap(), message.value.unwrap()) {
                    message.channel.unwrap().send((true, "".to_string())).unwrap();
                } else {
                    message.channel.unwrap().send((false, "".to_string())).unwrap();
                }
            }
            Operation::SetWithExpire => {
                if db.set_with_expire(message.key.unwrap(), message.value.unwrap(), message.deadline.unwrap()) {
                    message.channel.unwrap().send((true, "".to_string())).unwrap();
                } else {
                    message.channel.unwrap().send((false, "".to_string())).unwrap();
                }
            }
            Operation::Delete => {
                if db.delete(message.key.unwrap()) {
                    message.channel.unwrap().send((true, "".to_string())).unwrap();
                } else {
                    message.channel.unwrap().send((false, "".to_string())).unwrap();
                }
            }
            Operation::Clear => {
                if db.clear() {
                    message.channel.unwrap().send((true, "".to_string())).unwrap();
                } else {
                    message.channel.unwrap().send((false, "".to_string())).unwrap();
                }
            }
            Operation::Close => {
                panic!()
            }
        }
    }
}
