use axum::{routing::{get, post}, Json, Router};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, net::SocketAddr, sync::{Arc, Mutex}};
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

#[derive(Debug, Default)]
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
    let state = Arc::new(Mutex::new(AppState::default()));

    let app = Router::new()
        .route("/health", get(health))
        .route("/bots/register", post(register_bot))
        .route("/tasks/create", post(create_task))
        .route("/tasks/assign", post(assign_task))
        .route("/deliver", post(deliver))
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("orchestrator listening on {}", addr);
    axum::serve(tokio::net::TcpListener::bind(addr).await.unwrap(), app)
        .await
        .unwrap();
}

async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({"ok": true}))
}

async fn register_bot(
    state: axum::extract::State<Arc<Mutex<AppState>>>,
    Json(req): Json<RegisterBotRequest>,
) -> Json<RegisterBotResponse> {
    let id = Uuid::new_v4().to_string();
    let bot = Bot {
        id: id.clone(),
        name: req.name,
        capabilities: req.capabilities,
    };
    state.lock().unwrap().bots.insert(id.clone(), bot);
    Json(RegisterBotResponse { id })
}

async fn create_task(
    state: axum::extract::State<Arc<Mutex<AppState>>>,
    Json(req): Json<CreateTaskRequest>,
) -> Json<CreateTaskResponse> {
    let id = Uuid::new_v4().to_string();
    let task = Task {
        id: id.clone(),
        title: req.title,
        description: req.description,
        status: "open".to_string(),
        assignee: None,
    };
    state.lock().unwrap().tasks.insert(id.clone(), task);
    Json(CreateTaskResponse { id })
}

async fn assign_task(
    state: axum::extract::State<Arc<Mutex<AppState>>>,
    Json(req): Json<AssignTaskRequest>,
) -> Json<AssignTaskResponse> {
    let mut state = state.lock().unwrap();
    if let Some(task) = state.tasks.get_mut(&req.task_id) {
        task.assignee = Some(req.bot_id);
        task.status = "assigned".to_string();
        return Json(AssignTaskResponse { ok: true });
    }
    Json(AssignTaskResponse { ok: false })
}

async fn deliver(
    state: axum::extract::State<Arc<Mutex<AppState>>>,
    Json(req): Json<DeliverRequest>,
) -> Json<DeliverResponse> {
    let mut state = state.lock().unwrap();
    if let Some(task) = state.tasks.get_mut(&req.task_id) {
        task.status = format!("delivered: {}", req.summary);
        return Json(DeliverResponse { ok: true });
    }
    Json(DeliverResponse { ok: false })
}
