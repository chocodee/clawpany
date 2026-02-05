use axum::{
    extract::State,
    http::HeaderMap,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs,
    net::SocketAddr,
    path::Path,
    sync::{Arc, Mutex},
};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Bot {
    id: String,
    name: String,
    capabilities: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Task {
    id: String,
    title: String,
    description: String,
    status: String,
    assignee: Option<String>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct AppState {
    bots: HashMap<String, Bot>,
    tasks: HashMap<String, Task>,
}

#[derive(Debug, Deserialize)]
struct RegisterBotRequest {
    name: String,
    capabilities: Vec<String>,
}

#[derive(Debug, Serialize)]
struct RegisterBotResponse {
    id: String,
}

#[derive(Debug, Deserialize)]
struct CreateTaskRequest {
    title: String,
    description: String,
}

#[derive(Debug, Serialize)]
struct CreateTaskResponse {
    id: String,
}

#[derive(Debug, Deserialize)]
struct AssignTaskRequest {
    task_id: String,
    bot_id: String,
}

#[derive(Debug, Serialize)]
struct AssignTaskResponse {
    ok: bool,
}

#[derive(Debug, Deserialize)]
struct DeliverRequest {
    task_id: String,
    summary: String,
}

#[derive(Debug, Serialize)]
struct DeliverResponse {
    ok: bool,
}

#[tokio::main]
async fn main() {
    let state = Arc::new(Mutex::new(load_state("state.json")));

    let app = Router::new()
        .route("/health", get(health))
        .route("/bots/register", post(register_bot))
        .route("/tasks/create", post(create_task))
        .route("/tasks/assign", post(assign_task))
        .route("/deliver", post(deliver))
        .with_state(state.clone());

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("orchestrator listening on {}", addr);
    axum::serve(tokio::net::TcpListener::bind(addr).await.unwrap(), app)
        .await
        .unwrap();
}

async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({"ok": true}))
}

fn require_auth(headers: &HeaderMap) {
    let expected = std::env::var("ORCH_API_KEY").unwrap_or_else(|_| "dev_key".to_string());
    let auth = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    let token = auth.strip_prefix("Bearer ").unwrap_or("");
    if token != expected {
        panic!("unauthorized");
    }
}

fn load_state<P: AsRef<Path>>(path: P) -> AppState {
    if let Ok(data) = fs::read_to_string(&path) {
        if let Ok(state) = serde_json::from_str(&data) {
            return state;
        }
    }
    AppState::default()
}

fn save_state<P: AsRef<Path>>(path: P, state: &AppState) {
    if let Ok(data) = serde_json::to_string_pretty(state) {
        let _ = fs::write(path, data);
    }
}

async fn register_bot(
    State(state): State<Arc<Mutex<AppState>>>,
    headers: HeaderMap,
    Json(req): Json<RegisterBotRequest>,
) -> Json<RegisterBotResponse> {
    require_auth(&headers);
    let id = Uuid::new_v4().to_string();
    let bot = Bot {
        id: id.clone(),
        name: req.name,
        capabilities: req.capabilities,
    };
    let mut guard = state.lock().unwrap();
    guard.bots.insert(id.clone(), bot);
    save_state("state.json", &guard);
    Json(RegisterBotResponse { id })
}

async fn create_task(
    State(state): State<Arc<Mutex<AppState>>>,
    headers: HeaderMap,
    Json(req): Json<CreateTaskRequest>,
) -> Json<CreateTaskResponse> {
    require_auth(&headers);
    let id = Uuid::new_v4().to_string();
    let task = Task {
        id: id.clone(),
        title: req.title,
        description: req.description,
        status: "open".to_string(),
        assignee: None,
    };
    let mut guard = state.lock().unwrap();
    guard.tasks.insert(id.clone(), task);
    save_state("state.json", &guard);
    Json(CreateTaskResponse { id })
}

async fn assign_task(
    State(state): State<Arc<Mutex<AppState>>>,
    headers: HeaderMap,
    Json(req): Json<AssignTaskRequest>,
) -> Json<AssignTaskResponse> {
    require_auth(&headers);
    let mut guard = state.lock().unwrap();
    if let Some(task) = guard.tasks.get_mut(&req.task_id) {
        task.assignee = Some(req.bot_id);
        task.status = "assigned".to_string();
        save_state("state.json", &guard);
        return Json(AssignTaskResponse { ok: true });
    }
    Json(AssignTaskResponse { ok: false })
}

async fn deliver(
    State(state): State<Arc<Mutex<AppState>>>,
    headers: HeaderMap,
    Json(req): Json<DeliverRequest>,
) -> Json<DeliverResponse> {
    require_auth(&headers);
    let mut guard = state.lock().unwrap();
    if let Some(task) = guard.tasks.get_mut(&req.task_id) {
        task.status = format!("delivered: {}", req.summary);
        save_state("state.json", &guard);
        return Json(DeliverResponse { ok: true });
    }
    Json(DeliverResponse { ok: false })
}
