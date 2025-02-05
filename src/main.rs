use axum::{
    routing::post,
    Router, Json, extract::State,
};
use sqlx::{SqlitePool, FromRow};
use serde::{Deserialize, Serialize};
use web_push::{WebPushClient, WebPushMessage, WebPushError};
use web_push::vapid::{generate_vapid_keys, VapidKeys};
use tokio::sync::Mutex;
use std::sync::Arc;
use uuid::Uuid;
use dotenvy::dotenv;
use std::env;

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
    keys: String,  // Stored as JSON
    user_id: Option<String>,
}

// Define the shared state for the application (SQLite Pool)
type AppState = Arc<Mutex<SqlitePool>>;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").unwrap_or("sqlite:subscriptions.db".to_string());
    let pool = SqlitePool::connect(&database_url).await?;

    // Set up the Axum app
    let app = Router::new()
        .route("/subscribe", post(subscribe))
        .route("/send_notification", post(send_notification))
        .layer(State::new(Arc::new(Mutex::new(pool))));

    // Run the server
    axum::Server::bind(&"127.0.0.1:8080".parse()?)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

async fn subscribe(
    State(pool): State<AppState>,
    Json(subscription): Json<Subscription>,
) -> axum::response::Json<Subscription> {
    // Save subscription to the database
    let query = sqlx::query(
        "INSERT INTO subscriptions (endpoint, keys, user_id) VALUES (?, ?, ?)",
    )
    .bind(subscription.endpoint)
    .bind(serde_json::to_string(&subscription.keys).unwrap()) // Store keys as JSON
    .bind(subscription.user_id)
    .execute(pool.lock().await)
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
    Json(payload): Json<WebPushMessage>,
) -> axum::response::Json<String> {
    // Fetch all subscriptions from the database
    let subscriptions = sqlx::query_as::<_, StoredSubscription>(
        "SELECT id, endpoint, keys, user_id FROM subscriptions",
    )
    .fetch_all(pool.lock().await)
    .await;

    match subscriptions {
        Ok(subscriptions) => {
            let vapid_keys = get_vapid_keys();
            let client = WebPushClient::new(vapid_keys).unwrap();

            for sub in subscriptions {
                let keys: SubscriptionKeys = serde_json::from_str(&sub.keys).unwrap();

                let push_subscription = web_push::SubscriptionInfo {
                    endpoint: sub.endpoint,
                    p256dh: keys.p256dh,
                    auth: keys.auth,
                };

                let message = PushMessage {
                    payload: serde_json::to_string(&payload).unwrap(),
                    subscription: push_subscription,
                };

                match client.send(&message).await {
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

// VAPID key generation logic (can be replaced with your actual keys)
fn get_vapid_keys() -> VapidKeys {
    dotenv().ok();
    let public_key = env::var("VAPID_PUBLIC_KEY").unwrap();
    let private_key = env::var("VAPID_PRIVATE_KEY").unwrap();
    VapidKeys {
        public_key,
        private_key,
    }
}
