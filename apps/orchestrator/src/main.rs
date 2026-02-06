use axum::{
    Json, Router,
    extract::State,
    http::HeaderMap,
    routing::{get, post},
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
    project_id: String,
    title: String,
    description: String,
    status: String,
    assignee: Option<String>,
    #[serde(default)]
    delivery_summary: Option<String>,
}

const STATUS_OPEN: &str = "open";
const STATUS_ASSIGNED: &str = "assigned";
const STATUS_IN_PROGRESS: &str = "in_progress";
const STATUS_BLOCKED: &str = "blocked";
const STATUS_REVIEW: &str = "review";
const STATUS_DELIVERED: &str = "delivered";

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Client {
    id: String,
    name: String,
    contact: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Project {
    id: String,
    client_id: String,
    name: String,
    description: String,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct AppState {
    bots: HashMap<String, Bot>,
    tasks: HashMap<String, Task>,
    clients: HashMap<String, Client>,
    projects: HashMap<String, Project>,
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
struct CreateClientRequest {
    name: String,
    contact: String,
}

#[derive(Debug, Serialize)]
struct CreateClientResponse {
    id: String,
}

#[derive(Debug, Deserialize)]
struct CreateProjectRequest {
    client_id: String,
    name: String,
    description: String,
}

#[derive(Debug, Serialize)]
struct CreateProjectResponse {
    id: String,
}

#[derive(Debug, Deserialize)]
struct IntakeTaskRequest {
    project_id: String,
    title: String,
    description: String,
}

#[derive(Debug, Serialize)]
struct IntakeTaskResponse {
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
struct UpdateStatusRequest {
    task_id: String,
    status: String,
}

#[derive(Debug, Serialize)]
struct UpdateStatusResponse {
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
        .route("/clients/create", post(create_client))
        .route("/projects/create", post(create_project))
        .route("/tasks/intake", post(intake_task))
        .route("/tasks/assign", post(assign_task))
        .route("/tasks/status", post(update_status))
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
    if let Ok(data) = fs::read_to_string(&path)
        && let Ok(state) = serde_json::from_str(&data)
    {
        return state;
    }
    AppState::default()
}

fn save_state<P: AsRef<Path>>(path: P, state: &AppState) {
    if let Ok(data) = serde_json::to_string_pretty(state) {
        let _ = fs::write(path, data);
    }
}

fn normalize_status(status: &str) -> &str {
    if status.starts_with("delivered:") {
        STATUS_DELIVERED
    } else {
        status
    }
}

fn is_valid_status(status: &str) -> bool {
    matches!(
        normalize_status(status),
        STATUS_OPEN
            | STATUS_ASSIGNED
            | STATUS_IN_PROGRESS
            | STATUS_BLOCKED
            | STATUS_REVIEW
            | STATUS_DELIVERED
    )
}

fn can_transition(from: &str, to: &str) -> bool {
    let from = normalize_status(from);
    let to = normalize_status(to);
    match from {
        STATUS_OPEN => matches!(to, STATUS_ASSIGNED),
        STATUS_ASSIGNED => matches!(to, STATUS_IN_PROGRESS | STATUS_BLOCKED | STATUS_DELIVERED),
        STATUS_IN_PROGRESS => matches!(to, STATUS_BLOCKED | STATUS_REVIEW | STATUS_DELIVERED),
        STATUS_BLOCKED => matches!(to, STATUS_IN_PROGRESS | STATUS_REVIEW),
        STATUS_REVIEW => matches!(to, STATUS_IN_PROGRESS | STATUS_DELIVERED),
        STATUS_DELIVERED => false,
        _ => false,
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

async fn create_client(
    State(state): State<Arc<Mutex<AppState>>>,
    headers: HeaderMap,
    Json(req): Json<CreateClientRequest>,
) -> Json<CreateClientResponse> {
    require_auth(&headers);
    let id = Uuid::new_v4().to_string();
    let client = Client {
        id: id.clone(),
        name: req.name,
        contact: req.contact,
    };
    let mut guard = state.lock().unwrap();
    guard.clients.insert(id.clone(), client);
    save_state("state.json", &guard);
    Json(CreateClientResponse { id })
}

async fn create_project(
    State(state): State<Arc<Mutex<AppState>>>,
    headers: HeaderMap,
    Json(req): Json<CreateProjectRequest>,
) -> Json<CreateProjectResponse> {
    require_auth(&headers);
    let id = Uuid::new_v4().to_string();
    let project = Project {
        id: id.clone(),
        client_id: req.client_id,
        name: req.name,
        description: req.description,
    };
    let mut guard = state.lock().unwrap();
    guard.projects.insert(id.clone(), project);
    save_state("state.json", &guard);
    Json(CreateProjectResponse { id })
}

async fn intake_task(
    State(state): State<Arc<Mutex<AppState>>>,
    headers: HeaderMap,
    Json(req): Json<IntakeTaskRequest>,
) -> Json<IntakeTaskResponse> {
    require_auth(&headers);
    let id = Uuid::new_v4().to_string();
    let task = Task {
        id: id.clone(),
        project_id: req.project_id,
        title: req.title,
        description: req.description,
        status: STATUS_OPEN.to_string(),
        assignee: None,
        delivery_summary: None,
    };
    let mut guard = state.lock().unwrap();
    guard.tasks.insert(id.clone(), task);
    save_state("state.json", &guard);
    Json(IntakeTaskResponse { id })
}

async fn assign_task(
    State(state): State<Arc<Mutex<AppState>>>,
    headers: HeaderMap,
    Json(req): Json<AssignTaskRequest>,
) -> Json<AssignTaskResponse> {
    require_auth(&headers);
    let mut guard = state.lock().unwrap();
    if let Some(task) = guard.tasks.get_mut(&req.task_id) {
        if !can_transition(&task.status, STATUS_ASSIGNED) {
            return Json(AssignTaskResponse { ok: false });
        }
        task.assignee = Some(req.bot_id);
        task.status = STATUS_ASSIGNED.to_string();
        save_state("state.json", &guard);
        return Json(AssignTaskResponse { ok: true });
    }
    Json(AssignTaskResponse { ok: false })
}

async fn update_status(
    State(state): State<Arc<Mutex<AppState>>>,
    headers: HeaderMap,
    Json(req): Json<UpdateStatusRequest>,
) -> Json<UpdateStatusResponse> {
    require_auth(&headers);
    if !is_valid_status(&req.status) {
        return Json(UpdateStatusResponse { ok: false });
    }
    let mut guard = state.lock().unwrap();
    if let Some(task) = guard.tasks.get_mut(&req.task_id) {
        if !can_transition(&task.status, &req.status) {
            return Json(UpdateStatusResponse { ok: false });
        }
        task.status = req.status;
        save_state("state.json", &guard);
        return Json(UpdateStatusResponse { ok: true });
    }
    Json(UpdateStatusResponse { ok: false })
}

async fn deliver(
    State(state): State<Arc<Mutex<AppState>>>,
    headers: HeaderMap,
    Json(req): Json<DeliverRequest>,
) -> Json<DeliverResponse> {
    require_auth(&headers);
    let mut guard = state.lock().unwrap();
    if let Some(task) = guard.tasks.get_mut(&req.task_id) {
        if !can_transition(&task.status, STATUS_DELIVERED) {
            return Json(DeliverResponse { ok: false });
        }
        task.status = STATUS_DELIVERED.to_string();
        task.delivery_summary = Some(req.summary);
        save_state("state.json", &guard);
        return Json(DeliverResponse { ok: true });
    }
    Json(DeliverResponse { ok: false })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_statuses() {
        assert!(is_valid_status(STATUS_OPEN));
        assert!(is_valid_status(STATUS_ASSIGNED));
        assert!(is_valid_status(STATUS_IN_PROGRESS));
        assert!(is_valid_status(STATUS_BLOCKED));
        assert!(is_valid_status(STATUS_REVIEW));
        assert!(is_valid_status(STATUS_DELIVERED));
        assert!(is_valid_status("delivered: legacy"));
        assert!(!is_valid_status("unknown"));
    }

    #[test]
    fn transition_rules() {
        assert!(can_transition(STATUS_OPEN, STATUS_ASSIGNED));
        assert!(!can_transition(STATUS_OPEN, STATUS_DELIVERED));
        assert!(can_transition(STATUS_ASSIGNED, STATUS_IN_PROGRESS));
        assert!(can_transition(STATUS_ASSIGNED, STATUS_DELIVERED));
        assert!(can_transition(STATUS_IN_PROGRESS, STATUS_REVIEW));
        assert!(can_transition(STATUS_REVIEW, STATUS_DELIVERED));
        assert!(!can_transition(STATUS_DELIVERED, STATUS_REVIEW));
    }
}
