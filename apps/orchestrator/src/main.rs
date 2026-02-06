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
    lease_expires_at: Option<u64>,
    #[serde(default)]
    attempts: u32,
    #[serde(default)]
    last_error: Option<String>,
    #[serde(default)]
    delivery_summary: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Worker {
    id: String,
    name: String,
    capabilities: Vec<String>,
    last_heartbeat: u64,
}

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
    workers: HashMap<String, Worker>,
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
struct RegisterWorkerRequest {
    name: String,
    capabilities: Vec<String>,
}

#[derive(Debug, Serialize)]
struct RegisterWorkerResponse {
    id: String,
}

#[derive(Debug, Deserialize)]
struct WorkerHeartbeatRequest {
    worker_id: String,
}

#[derive(Debug, Serialize)]
struct WorkerHeartbeatResponse {
    ok: bool,
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
struct ClaimTaskRequest {
    worker_id: String,
}

#[derive(Debug, Serialize)]
struct ClaimTaskResponse {
    ok: bool,
    task: Option<Task>,
}

#[derive(Debug, Deserialize)]
struct CompleteTaskRequest {
    task_id: String,
    worker_id: String,
    summary: String,
}

#[derive(Debug, Serialize)]
struct CompleteTaskResponse {
    ok: bool,
}

#[derive(Debug, Deserialize)]
struct FailTaskRequest {
    task_id: String,
    worker_id: String,
    error: String,
}

#[derive(Debug, Serialize)]
struct FailTaskResponse {
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
        .route("/workers/register", post(register_worker))
        .route("/workers/heartbeat", post(worker_heartbeat))
        .route("/clients/create", post(create_client))
        .route("/projects/create", post(create_project))
        .route("/tasks/intake", post(intake_task))
        .route("/tasks/assign", post(assign_task))
        .route("/tasks/claim", post(claim_task))
        .route("/tasks/complete", post(complete_task))
        .route("/tasks/fail", post(fail_task))
        .route("/tasks", get(list_tasks))
        .route("/clients", get(list_clients))
        .route("/projects", get(list_projects))
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

fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
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

async fn register_worker(
    State(state): State<Arc<Mutex<AppState>>>,
    headers: HeaderMap,
    Json(req): Json<RegisterWorkerRequest>,
) -> Json<RegisterWorkerResponse> {
    require_auth(&headers);
    let id = Uuid::new_v4().to_string();
    let worker = Worker {
        id: id.clone(),
        name: req.name,
        capabilities: req.capabilities,
        last_heartbeat: now_secs(),
    };
    let mut guard = state.lock().unwrap();
    guard.workers.insert(id.clone(), worker);
    save_state("state.json", &guard);
    Json(RegisterWorkerResponse { id })
}

async fn worker_heartbeat(
    State(state): State<Arc<Mutex<AppState>>>,
    headers: HeaderMap,
    Json(req): Json<WorkerHeartbeatRequest>,
) -> Json<WorkerHeartbeatResponse> {
    require_auth(&headers);
    let mut guard = state.lock().unwrap();
    if let Some(worker) = guard.workers.get_mut(&req.worker_id) {
        worker.last_heartbeat = now_secs();
        save_state("state.json", &guard);
        return Json(WorkerHeartbeatResponse { ok: true });
    }
    Json(WorkerHeartbeatResponse { ok: false })
}

async fn list_clients(
    State(state): State<Arc<Mutex<AppState>>>,
    headers: HeaderMap,
) -> Json<Vec<Client>> {
    require_auth(&headers);
    let guard = state.lock().unwrap();
    let mut clients: Vec<Client> = guard.clients.values().cloned().collect();
    clients.sort_by(|a, b| a.name.cmp(&b.name));
    Json(clients)
}

async fn list_projects(
    State(state): State<Arc<Mutex<AppState>>>,
    headers: HeaderMap,
) -> Json<Vec<Project>> {
    require_auth(&headers);
    let guard = state.lock().unwrap();
    let mut projects: Vec<Project> = guard.projects.values().cloned().collect();
    projects.sort_by(|a, b| a.name.cmp(&b.name));
    Json(projects)
}

async fn list_tasks(
    State(state): State<Arc<Mutex<AppState>>>,
    headers: HeaderMap,
) -> Json<Vec<Task>> {
    require_auth(&headers);
    let guard = state.lock().unwrap();
    let mut tasks: Vec<Task> = guard.tasks.values().cloned().collect();
    tasks.sort_by(|a, b| a.title.cmp(&b.title));
    Json(tasks)
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
        status: "open".to_string(),
        assignee: None,
        lease_expires_at: None,
        attempts: 0,
        last_error: None,
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
        task.assignee = Some(req.bot_id);
        task.status = "assigned".to_string();
        task.lease_expires_at = Some(now_secs() + 300);
        save_state("state.json", &guard);
        return Json(AssignTaskResponse { ok: true });
    }
    Json(AssignTaskResponse { ok: false })
}

async fn claim_task(
    State(state): State<Arc<Mutex<AppState>>>,
    headers: HeaderMap,
    Json(req): Json<ClaimTaskRequest>,
) -> Json<ClaimTaskResponse> {
    require_auth(&headers);
    let mut guard = state.lock().unwrap();
    let now = now_secs();
    let mut selected: Option<Task> = None;
    for task in guard.tasks.values_mut() {
        let expired = task.lease_expires_at.map(|ts| ts <= now).unwrap_or(true);
        if (task.status == "open" || task.status == "assigned") && expired {
            task.status = "in_progress".to_string();
            task.assignee = Some(req.worker_id.clone());
            task.lease_expires_at = Some(now + 300);
            task.attempts = task.attempts.saturating_add(1);
            selected = Some(task.clone());
            break;
        }
    }
    if selected.is_some() {
        save_state("state.json", &guard);
        return Json(ClaimTaskResponse {
            ok: true,
            task: selected,
        });
    }
    Json(ClaimTaskResponse {
        ok: false,
        task: None,
    })
}

async fn complete_task(
    State(state): State<Arc<Mutex<AppState>>>,
    headers: HeaderMap,
    Json(req): Json<CompleteTaskRequest>,
) -> Json<CompleteTaskResponse> {
    require_auth(&headers);
    let mut guard = state.lock().unwrap();
    if let Some(task) = guard.tasks.get_mut(&req.task_id) {
        if task.assignee.as_deref() != Some(&req.worker_id) {
            return Json(CompleteTaskResponse { ok: false });
        }
        task.status = "delivered".to_string();
        task.delivery_summary = Some(req.summary);
        task.lease_expires_at = None;
        save_state("state.json", &guard);
        return Json(CompleteTaskResponse { ok: true });
    }
    Json(CompleteTaskResponse { ok: false })
}

async fn fail_task(
    State(state): State<Arc<Mutex<AppState>>>,
    headers: HeaderMap,
    Json(req): Json<FailTaskRequest>,
) -> Json<FailTaskResponse> {
    require_auth(&headers);
    let mut guard = state.lock().unwrap();
    if let Some(task) = guard.tasks.get_mut(&req.task_id) {
        if task.assignee.as_deref() != Some(&req.worker_id) {
            return Json(FailTaskResponse { ok: false });
        }
        task.status = "failed".to_string();
        task.last_error = Some(req.error);
        task.lease_expires_at = None;
        save_state("state.json", &guard);
        return Json(FailTaskResponse { ok: true });
    }
    Json(FailTaskResponse { ok: false })
}

async fn deliver(
    State(state): State<Arc<Mutex<AppState>>>,
    headers: HeaderMap,
    Json(req): Json<DeliverRequest>,
) -> Json<DeliverResponse> {
    require_auth(&headers);
    let mut guard = state.lock().unwrap();
    if let Some(task) = guard.tasks.get_mut(&req.task_id) {
        task.status = "delivered".to_string();
        task.delivery_summary = Some(req.summary);
        task.lease_expires_at = None;
        save_state("state.json", &guard);
        return Json(DeliverResponse { ok: true });
    }
    Json(DeliverResponse { ok: false })
}
