use axum::{
    extract::State,
    http::{StatusCode, Uri},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router, ServiceExt,
};
use dotenvy::dotenv;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqlitePool};
use std::{env::var, fs::File, sync::Arc};
use tokio::net::TcpListener;
use tower_http::services::{ServeDir, ServeFile};
use web_push::{
    IsahcWebPushClient, SubscriptionInfo, VapidSignatureBuilder, WebPushClient,
    WebPushMessageBuilder,
};

#[derive(Serialize, Deserialize)]
struct Subscription {
    endpoint: String,
    keys: SubscriptionKeys,
    user_id: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct SubscriptionKeys {
    p256dh: String,
    auth: String,
}

#[derive(FromRow, Debug)]
struct StoredSubscription {
    id: i64,
    endpoint: String,
    keys: String, // Stored as JSON
    user_id: Option<String>,
}

pub struct StaticFile<T>(pub T);

type AppState = Arc<SqlitePool>;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let database_url = var("DATABASE_URL").unwrap_or("sqlite:subscriptions.db".to_string());
    let pool = SqlitePool::connect(&database_url).await?;

    let servedir = ServeDir::new("frontend");
    let servefile = ServeFile::new("public.b64");

    let app = Router::new()
        .fallback_service(servedir)
        .nest_service("/pubkey", servefile)
        .route("/subscribe", post(subscribe))
        .route("/send_notification", post(send_notification))
        .with_state(Arc::new(pool));

    let listener = TcpListener::bind("0.0.0.0:8080").await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn subscribe(
    State(pool): State<AppState>,
    Json(subscription): Json<Subscription>,
) -> axum::response::Json<Subscription> {
    let query = sqlx::query("INSERT INTO subscriptions (endpoint, keys, user_id) VALUES (?, ?, ?)")
        .bind(&subscription.endpoint)
        .bind(serde_json::to_string(&subscription.keys).unwrap()) // Store keys as JSON
        .bind(&subscription.user_id)
        .execute(&*pool)
        .await;

    match query {
        Ok(_) => axum::response::Json(subscription),
        Err(e) => {
            eprintln!("Error saving subscription: {:?}", e);
            axum::response::Json(Subscription {
                endpoint: "".to_string(),
                keys: SubscriptionKeys {
                    p256dh: "".to_string(),
                    auth: "".to_string(),
                },
                user_id: None,
            })
        }
    }
}

async fn send_notification(
    State(pool): State<AppState>,
    Json(()): Json<()>,
) -> axum::response::Json<String> {
    let subscriptions = sqlx::query_as::<_, StoredSubscription>(
        "SELECT id, endpoint, keys, user_id FROM subscriptions",
    )
    .fetch_all(&*pool)
    .await;

    match subscriptions {
        Ok(subscriptions) => {
            let client = IsahcWebPushClient::new().unwrap();

            for sub in subscriptions {
                let keys: SubscriptionKeys = serde_json::from_str(&sub.keys).unwrap();

                let push_subscription =
                    SubscriptionInfo::new(&sub.endpoint, &keys.p256dh, &keys.auth);

                let file = File::open("private.pem").unwrap();
                let sig_builder = VapidSignatureBuilder::from_pem(file, &push_subscription)
                    .unwrap()
                    .build()
                    .unwrap();

                let mut builder = WebPushMessageBuilder::new(&push_subscription);
                builder.set_payload(web_push::ContentEncoding::default(), b"some body to send");
                builder.set_vapid_signature(sig_builder);

                let message = builder.build().unwrap();

                match client.send(message).await {
                    Ok(_) => println!("Notification sent to: {}", sub.endpoint),
                    Err(err) => eprintln!("Error sending notification: {:?}", err),
                }
            }

            axum::response::Json("Notifications sent".to_string())
        }
        Err(e) => {
            eprintln!("Error fetching subscriptions: {:?}", e);
            axum::response::Json("Error fetching subscriptions".to_string())
        }
    }
}
