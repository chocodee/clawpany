import fetch from "node-fetch";

const OPENCLAW_URL = process.env.OPENCLAW_URL || "http://localhost:3001";
const OPENCLAW_API_KEY = process.env.OPENCLAW_API_KEY || "";

export async function runOpenClaw(prompt) {
  const res = await fetch(`${OPENCLAW_URL}/agent/run`, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      ...(OPENCLAW_API_KEY ? { Authorization: `Bearer ${OPENCLAW_API_KEY}` } : {}),
    },
    body: JSON.stringify({ message: prompt }),
  });

  if (!res.ok) {
    throw new Error(`OpenClaw error: ${res.status}`);
  }

  const data = await res.json();
  return data?.message || "OpenClaw completed task.";
}
