use std::env;
use std::sync::LazyLock;

use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

pub static API_URL: LazyLock<String> = LazyLock::new(|| {
    env::var("ARRABBIATA_API_URL").expect("ARRABBIATA_API_URL must be set")
});

pub static USER_ID: LazyLock<String> = LazyLock::new(|| {
    env::var("ARRABBIATA_USER_ID").expect("ARRABBIATA_USER_ID must be set")
});

pub static FALLBACK_USER_ID: LazyLock<String> = LazyLock::new(|| {
    env::var("ARRABBIATA_FALLBACK_USER_ID").expect("ARRABBIATA_FALLBACK_USER_ID must be set")
});

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiRequest {
    pub user_id: String,
    pub workout_type: Option<i32>,
    pub planned_time: Option<i64>,
    pub actual_time: Option<i64>,
    pub workout_date: Option<String>,
}

#[derive(Deserialize, Default)]
#[serde(default)]
pub struct ApiResponse {
    pub workout: Option<Workout>,
    pub stats: Option<Stats>,
    pub workouts: Option<Vec<f64>>,
}

#[derive(Deserialize)]
pub struct Workout {
    #[serde(alias = "userId", alias = "UserId", alias = "user_id")]
    pub user_id: Option<String>,
    #[serde(alias = "plannedTime", alias = "PlannedTime", alias = "planned_time")]
    pub planned_time: Option<f64>,
    #[serde(alias = "workoutType", alias = "WorkoutType", alias = "workout_type")]
    pub workout_type: Option<i32>,
}

#[derive(Deserialize)]
pub struct Stats {
    #[serde(alias = "totalRuns", alias = "TotalRuns", alias = "total_runs")]
    pub total_runs: Option<u64>,
    #[serde(alias = "workCount", alias = "WorkCount", alias = "work_count")]
    pub work_count: Option<f64>,
    #[serde(alias = "pauseCount", alias = "PauseCount", alias = "pause_count")]
    pub pause_count: Option<f64>,
}

pub enum ApiResult {
    Success(ApiResponse),
    Error(String),
}

pub fn spawn_request(
    client: &reqwest::Client,
    tx: &mpsc::UnboundedSender<ApiResult>,
    req: ApiRequest,
) {
    let client = client.clone();
    let tx = tx.clone();
    tokio::spawn(async move {
        let result: Result<ApiResponse, String> = async {
            let resp = client
                .post(API_URL.as_str())
                .json(&req)
                .send()
                .await
                .map_err(|e| e.to_string())?;
            let status = resp.status();
            let body = resp.text().await.map_err(|e| e.to_string())?;
            if !status.is_success() {
                return Err(format!("HTTP {status}: {body}"));
            }
            serde_json::from_str(&body).map_err(|e| format!("{e} -- response: {body}"))
        }
        .await;
        let _ = match result {
            Ok(data) => tx.send(ApiResult::Success(data)),
            Err(e) => tx.send(ApiResult::Error(e)),
        };
    });
}
