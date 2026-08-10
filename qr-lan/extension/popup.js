// Duplicated in `companion/src/main.rs` — deliberately, so neither side needs
// a config file.
const PORT = 48213;

const app = document.getElementById("app");

function showStatus(text) {
  app.replaceChildren(Object.assign(document.createElement("p"), { className: "status", textContent: text }));
}

function show(url, svgSource) {
  // Parsed as XML rather than assigned through innerHTML: the SVG is data from
  // the companion, not markup for this page.
  const svg = new DOMParser().parseFromString(svgSource, "image/svg+xml").documentElement;
  const frame = document.createElement("div");
  frame.className = "qr";
  frame.append(svg);

  const label = document.createElement("button");
  label.className = "url";
  label.textContent = url;
  label.title = "Copy";
  label.addEventListener("click", async () => {
    await navigator.clipboard.writeText(url);
    label.textContent = "copied";
    setTimeout(() => (label.textContent = url), 800);
  });

  app.replaceChildren(frame, label);
}

async function main() {
  const [tab] = await chrome.tabs.query({ active: true, currentWindow: true });
  const url = tab?.url ?? "";
  if (!/^https?:/.test(url)) {
    showStatus("Can't share this page");
    return;
  }

  let response;
  try {
    response = await fetch(`http://127.0.0.1:${PORT}/qr?url=${encodeURIComponent(url)}`);
  } catch {
    showStatus("qr-lan companion is not running");
    return;
  }

  const body = await response.json().catch(() => null);
  if (!response.ok || !body?.svg) {
    showStatus(body?.error ?? `Companion returned ${response.status}`);
    return;
  }
  show(body.url, body.svg);
}

main();
