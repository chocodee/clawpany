import fetch from "node-fetch";

const ORCH_URL = process.env.ORCH_URL || "http://localhost:3000";
const ORCH_API_KEY = process.env.ORCH_API_KEY || "dev_key";
const WORKER_NAME = process.env.WORKER_NAME || "worker-1";
const POLL_INTERVAL_MS = Number(process.env.POLL_INTERVAL_MS || 5000);

const LLM_PROVIDER = process.env.LLM_PROVIDER || "echo";
const LLM_MODEL = process.env.LLM_MODEL || "";
const LLM_BIN = process.env.LLM_BIN || "";

const headers = {
  "Content-Type": "application/json",
  Authorization: `Bearer ${ORCH_API_KEY}`,
};

async function registerBot() {
  const res = await fetch(`${ORCH_URL}/bots/register`, {
    method: "POST",
    headers,
    body: JSON.stringify({
      name: WORKER_NAME,
      capabilities: ["general"],
    }),
  });
  const data = await res.json();
  return data.id;
}

async function listTasks() {
  const res = await fetch(`${ORCH_URL}/tasks`, { headers });
  return res.json();
}

async function assignTask(taskId, botId) {
  const res = await fetch(`${ORCH_URL}/tasks/assign`, {
    method: "POST",
    headers,
    body: JSON.stringify({ task_id: taskId, bot_id: botId }),
  });
  return res.json();
}

async function deliverTask(taskId, summary) {
  const res = await fetch(`${ORCH_URL}/deliver`, {
    method: "POST",
    headers,
    body: JSON.stringify({ task_id: taskId, summary }),
  });
  return res.json();
}

async function runCommand(cmd, args) {
  const { spawn } = await import("node:child_process");
  return new Promise((resolve, reject) => {
    const child = spawn(cmd, args, { stdio: ["ignore", "pipe", "pipe"] });
    let out = "";
    let err = "";
    child.stdout.on("data", (d) => (out += d.toString()));
    child.stderr.on("data", (d) => (err += d.toString()));
    child.on("error", reject);
    child.on("close", (code) => {
      if (code === 0) resolve(out.trim());
      else reject(new Error(err || `Command failed: ${code}`));
    });
  });
}

function buildPrompt(task) {
  return `Task: ${task.title}\n\nDescription: ${task.description}\n\nProvide a short completion summary.`;
}

async function workOnTask(task) {
  const prompt = buildPrompt(task);
  let summary = `Worker ${WORKER_NAME} completed task ${task.id}`;

  try {
    if (LLM_PROVIDER === "ollama") {
      summary = await runCommand(LLM_BIN || "ollama", ["run", LLM_MODEL, prompt]);
    } else if (LLM_PROVIDER === "llamacpp") {
      summary = await runCommand(LLM_BIN || "./main", ["-m", LLM_MODEL, "-p", prompt]);
    } else {
      summary = `${summary}. (echo mode)`;
    }
  } catch (err) {
    summary = `${summary}. (llm error: ${err.message})`;
  }

  await deliverTask(task.id, summary);
}

async function loop(botId) {
  while (true) {
    try {
      const tasks = await listTasks();
      const open = tasks.find((t) => t.status === "open");
      if (open) {
        const assigned = await assignTask(open.id, botId);
        if (assigned.ok) {
          await workOnTask(open);
        }
      }
    } catch (err) {
      console.error("worker error", err);
    }
    await new Promise((r) => setTimeout(r, POLL_INTERVAL_MS));
  }
}

const botId = await registerBot();
console.log(`registered bot ${botId}`);
await loop(botId);
