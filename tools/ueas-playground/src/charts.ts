/**
 * XSS-safe pure-SVG telemetry charts (empirical Big-O scaling & cache hit rates).
 */

export interface ScalePoint {
  n: number;
  worst_steps: number;
  worst_peak_bytes: number;
}

const SVG_NS = "http://www.w3.org/2000/svg";

export function renderComplexityChart(container: HTMLElement, points: ScalePoint[], label: string): void {
  container.replaceChildren();
  const title = document.createElement("div");
  title.className = "chart-title";
  title.textContent = `Empirical Step Scaling (${label})`;
  container.appendChild(title);

  if (points.length === 0) {
    const empty = document.createElement("div");
    empty.className = "chart-empty";
    empty.textContent = "Run Verify (L3) to plot empirical Big-O scaling.";
    container.appendChild(empty);
    return;
  }

  const w = 360;
  const h = 150;
  const pad = 24;
  const svg = document.createElementNS(SVG_NS, "svg");
  svg.setAttribute("viewBox", `0 0 ${w} ${h}`);
  svg.setAttribute("class", "chart-svg");

  const maxN = Math.max(...points.map((p) => p.n), 1);
  const maxS = Math.max(...points.map((p) => p.worst_steps), 1);

  const coords = points.map((p) => {
    const x = pad + ((p.n / maxN) * (w - 2 * pad));
    const y = h - pad - ((p.worst_steps / maxS) * (h - 2 * pad));
    return `${x.toFixed(1)},${y.toFixed(1)}`;
  });

  const polyline = document.createElementNS(SVG_NS, "polyline");
  polyline.setAttribute("fill", "none");
  polyline.setAttribute("stroke", "#38bdf8");
  polyline.setAttribute("stroke-width", "2.5");
  polyline.setAttribute("points", coords.join(" "));
  svg.appendChild(polyline);

  for (const pt of coords) {
    const [cx, cy] = pt.split(",");
    const dot = document.createElementNS(SVG_NS, "circle");
    dot.setAttribute("cx", cx);
    dot.setAttribute("cy", cy);
    dot.setAttribute("r", "3.5");
    dot.setAttribute("fill", "#facc15");
    svg.appendChild(dot);
  }

  container.appendChild(svg);
}
