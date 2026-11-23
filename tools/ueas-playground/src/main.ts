import { renderComplexityChart, type ScalePoint } from "./charts";
import { STUDIO_EXAMPLES } from "./examples";

interface WorkerReply {
  id: number;
  ok: boolean;
  payload?: string;
  error?: string;
}

const editor = document.getElementById("editor") as HTMLTextAreaElement;
const output = document.getElementById("output") as HTMLPreElement;
const targetSelect = document.getElementById("target-select") as HTMLSelectElement;
const exampleSelect = document.getElementById("example-select") as HTMLSelectElement;
const chartPanel = document.getElementById("chart-panel") as HTMLDivElement;

for (const ex of STUDIO_EXAMPLES) {
  const opt = document.createElement("option");
  opt.value = ex.id;
  opt.textContent = ex.label;
  exampleSelect.appendChild(opt);
}

const hashCode = location.hash.startsWith("#code=")
  ? decodeURIComponent(location.hash.slice(6))
  : null;
editor.value = hashCode ?? STUDIO_EXAMPLES[0].source;
renderComplexityChart(chartPanel, [], "none");

exampleSelect.addEventListener("change", () => {
  const found = STUDIO_EXAMPLES.find((e) => e.id === exampleSelect.value);
  if (found) {
    editor.value = found.source;
  }
});

const worker = new Worker(new URL("./worker.ts", import.meta.url), { type: "module" });
let nextId = 1;
let lastCertJson: string | null = null;
const pending = new Map<number, (reply: WorkerReply) => void>();

worker.onmessage = (event: MessageEvent<WorkerReply>) => {
  const cb = pending.get(event.data.id);
  if (cb) {
    pending.delete(event.data.id);
    cb(event.data);
  }
};

function callWorker(
  action: "check" | "verify" | "transpile" | "run" | "format",
  extra: { target?: string; level?: string } = {},
): Promise<WorkerReply> {
  return new Promise((resolve) => {
    const id = nextId++;
    pending.set(id, resolve);
    worker.postMessage({ id, action, source: editor.value, ...extra });
  });
}

function renderOutput(text: string): void {
  output.textContent = text;
}

document.getElementById("btn-run")?.addEventListener("click", async () => {
  const res = await callWorker("run");
  renderOutput(res.ok ? JSON.stringify(JSON.parse(res.payload ?? "{}"), null, 2) : `Error: ${res.error}`);
});

document.getElementById("btn-verify")?.addEventListener("click", async () => {
  const res = await callWorker("verify", { level: "L3" });
  if (res.ok && res.payload) {
    lastCertJson = res.payload;
    const cert = JSON.parse(res.payload) as {
      complexity_reports?: Array<{ fitted_time?: string; measurements?: ScalePoint[] }>;
    };
    const first = cert.complexity_reports?.[0];
    renderComplexityChart(chartPanel, first?.measurements ?? [], first?.fitted_time ?? "none");
    renderOutput(JSON.stringify(cert, null, 2));
  } else {
    renderOutput(`Error: ${res.error}`);
  }
});

document.getElementById("btn-fmt")?.addEventListener("click", async () => {
  const res = await callWorker("format");
  if (res.ok && res.payload) {
    const parsed = JSON.parse(res.payload) as { ok: boolean; formatted?: string };
    if (parsed.ok && parsed.formatted) {
      editor.value = parsed.formatted;
      renderOutput("Formatted cleanly.");
    } else {
      renderOutput(JSON.stringify(parsed, null, 2));
    }
  } else {
    renderOutput(`Error: ${res.error}`);
  }
});

document.getElementById("btn-transpile")?.addEventListener("click", async () => {
  const res = await callWorker("transpile", { target: targetSelect.value });
  if (res.ok && res.payload) {
    const parsed = JSON.parse(res.payload) as { ok: boolean; artifact?: { code: string; test_code?: string } };
    if (parsed.ok && parsed.artifact) {
      const combined = parsed.artifact.test_code
        ? `${parsed.artifact.code}\n${parsed.artifact.test_code}`
        : parsed.artifact.code;
      renderOutput(combined);
    } else {
      renderOutput(JSON.stringify(parsed, null, 2));
    }
  } else {
    renderOutput(`Error: ${res.error}`);
  }
});

document.getElementById("btn-share")?.addEventListener("click", () => {
  location.hash = `code=${encodeURIComponent(editor.value)}`;
  renderOutput("Updated shareable URL hash (#code=...).");
});

document.getElementById("btn-cert")?.addEventListener("click", () => {
  if (!lastCertJson) {
    renderOutput("Run Verify (L3) first to generate a SHA-256 certificate.");
    return;
  }
  const blob = new Blob([lastCertJson], { type: "application/json" });
  const url = URL.createObjectURL(blob);
  const a = document.createElement("a");
  a.href = url;
  a.download = "algorithm.ueas.cert.json";
  a.click();
  URL.revokeObjectURL(url);
});
