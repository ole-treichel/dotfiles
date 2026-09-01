// Duplicated in `companion/src/main.rs` — deliberately, so neither side needs
// a config file.
const PORT = 48213;

// Firefox exposes promise-returning APIs on `browser` and keeps `chrome`
// callback-only, so `await chrome.tabs.query(...)` yields undefined there.
// Chrome only grew `browser` in 148. Taking whichever exists covers both
// without a polyfill for the single call this popup makes.
const ext = globalThis.browser ?? globalThis.chrome;

const app = document.getElementById("app");
const errorView = app.querySelector('[data-view="error"]');
const qrFrame = app.querySelector(".qr");
const urlButton = app.querySelector(".url");

function showError(text) {
  errorView.textContent = text;
  app.dataset.state = "error";
}

function showQr(url, svgSource) {
  // Parsed as XML rather than assigned through innerHTML: the SVG is data from
  // the companion, not markup for this page.
  const svg = new DOMParser().parseFromString(svgSource, "image/svg+xml").documentElement;
  qrFrame.replaceChildren(svg);

  urlButton.textContent = url;
  urlButton.onclick = async () => {
    await navigator.clipboard.writeText(url);
    urlButton.textContent = "copied";
    setTimeout(() => (urlButton.textContent = url), 800);
  };

  app.dataset.state = "qr";
}

async function main() {
  const [tab] = await ext.tabs.query({ active: true, currentWindow: true });
  const url = tab?.url ?? "";
  if (!/^https?:/.test(url)) {
    showError("Can't share this page");
    return;
  }

  let response;
  try {
    response = await fetch(`http://127.0.0.1:${PORT}/qr?url=${encodeURIComponent(url)}`);
  } catch {
    showError("qr-lan companion is not running");
    return;
  }

  const body = await response.json().catch(() => null);
  if (!response.ok || !body?.svg) {
    showError(body?.error ?? `Companion returned ${response.status}`);
    return;
  }
  showQr(body.url, body.svg);
}

// Belt and suspenders: any exception anywhere above must still land on
// screen as text, never as a silent failure that leaves the popup blank.
main().catch((err) => showError(err?.message || "Something went wrong"));
