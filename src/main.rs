use std::env;
use std::net::SocketAddr;

use axum::Router;
use axum::extract::Json;
use axum::response::Html;
use axum::routing::{get, post};
use leptos::prelude::*;
use nialang::driver::pipeline::compile_to_ll;
use serde::{Deserialize, Serialize};

const DEFAULT_SOURCE: &str = r#"fn main() i32 {
    let a: i32 = 1;
    let b = 2;
    a + b
}
"#;

const STYLE: &str = r#"
:root {
    color-scheme: dark;
    --bg: #101214;
    --panel: #181b1f;
    --panel-strong: #20242a;
    --border: #333941;
    --text: #f4f6f8;
    --muted: #9aa4af;
    --accent: #6ee7b7;
    --accent-strong: #34d399;
    --danger: #ff8a8a;
    --danger-bg: #2b1719;
    --mono: "SFMono-Regular", "Cascadia Code", "Liberation Mono", Menlo, Consolas, monospace;
    --sans: Inter, ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
}

* {
    box-sizing: border-box;
}

html,
body {
    min-height: 100%;
    margin: 0;
}

body {
    background: var(--bg);
    color: var(--text);
    font-family: var(--sans);
}

.app {
    display: grid;
    grid-template-rows: auto minmax(0, 1fr);
    min-height: 100vh;
}

.topbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
    min-height: 58px;
    padding: 0 18px;
    border-bottom: 1px solid var(--border);
    background: #13161a;
}

.brand {
    display: inline-flex;
    align-items: center;
    gap: 10px;
    min-width: 0;
    font-weight: 700;
}

.brand-mark {
    display: grid;
    place-items: center;
    width: 30px;
    height: 30px;
    border: 1px solid #4b5563;
    background: #20242a;
    color: var(--accent);
    font-family: var(--mono);
    font-size: 15px;
    line-height: 1;
}

.toolbar {
    display: inline-flex;
    align-items: center;
    gap: 10px;
    min-width: 0;
}

.status {
    color: var(--muted);
    font-size: 13px;
    white-space: nowrap;
}

.status.ok {
    color: var(--accent);
}

.status.error {
    color: var(--danger);
}

.compile-button {
    min-height: 34px;
    padding: 0 14px;
    border: 1px solid #48cfa4;
    border-radius: 6px;
    background: var(--accent-strong);
    color: #071411;
    font: 700 14px var(--sans);
    cursor: pointer;
}

.compile-button:disabled {
    cursor: wait;
    opacity: 0.7;
}

.workspace {
    display: grid;
    grid-template-columns: minmax(280px, 1fr) minmax(280px, 1fr);
    min-height: 0;
}

.pane {
    display: grid;
    grid-template-rows: auto minmax(0, 1fr);
    min-width: 0;
    min-height: 0;
    border-right: 1px solid var(--border);
    background: var(--panel);
}

.pane:last-child {
    border-right: 0;
}

.pane-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    min-height: 42px;
    padding: 0 14px;
    border-bottom: 1px solid var(--border);
    background: var(--panel-strong);
}

.pane-title {
    overflow: hidden;
    color: #d9dee5;
    font-size: 13px;
    font-weight: 700;
    text-overflow: ellipsis;
    white-space: nowrap;
}

.pane-meta {
    color: var(--muted);
    font: 12px var(--mono);
    white-space: nowrap;
}

.editor,
.output {
    width: 100%;
    height: 100%;
    min-height: 0;
    margin: 0;
    padding: 16px;
    border: 0;
    outline: 0;
    resize: none;
    background: #111418;
    color: var(--text);
    font: 14px/1.55 var(--mono);
    tab-size: 4;
}

.editor {
    caret-color: var(--accent);
}

.output {
    overflow: auto;
    white-space: pre;
}

.output.error {
    background: var(--danger-bg);
    color: #ffd6d6;
}

@media (max-width: 840px) {
    .topbar {
        align-items: stretch;
        flex-direction: column;
        padding: 12px;
    }

    .toolbar {
        justify-content: space-between;
        width: 100%;
    }

    .workspace {
        grid-template-columns: 1fr;
        grid-template-rows: minmax(360px, 1fr) minmax(360px, 1fr);
    }

    .pane {
        border-right: 0;
        border-bottom: 1px solid var(--border);
    }

    .pane:last-child {
        border-bottom: 0;
    }
}
"#;

const CLIENT_JS: &str = r##"
const source = document.querySelector("#source");
const output = document.querySelector("#output");
const statusLine = document.querySelector("#status");
const compileButton = document.querySelector("#compile");
const lineCount = document.querySelector("#line-count");
let debounce = null;
let requestId = 0;

function updateLineCount() {
    const lines = source.value.length === 0 ? 1 : source.value.split("\n").length;
    lineCount.textContent = `${lines} lines`;
}

async function compileNow() {
    const currentRequest = ++requestId;
    compileButton.disabled = true;
    statusLine.textContent = "Compiling";
    statusLine.className = "status";

    try {
        const response = await fetch("/api/compile", {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({ source: source.value }),
        });

        const payload = await response.json();
        if (currentRequest !== requestId) {
            return;
        }

        output.textContent = payload.output;
        if (payload.status === "ok") {
            output.className = "output";
            statusLine.textContent = "Compiled";
            statusLine.className = "status ok";
        } else {
            output.className = "output error";
            statusLine.textContent = "Error";
            statusLine.className = "status error";
        }
    } catch (error) {
        if (currentRequest !== requestId) {
            return;
        }

        output.textContent = `request failed: ${error}`;
        output.className = "output error";
        statusLine.textContent = "Request failed";
        statusLine.className = "status error";
    } finally {
        if (currentRequest === requestId) {
            compileButton.disabled = false;
        }
    }
}

function scheduleCompile() {
    clearTimeout(debounce);
    updateLineCount();
    statusLine.textContent = "Edited";
    statusLine.className = "status";
    debounce = setTimeout(compileNow, 450);
}

source.addEventListener("input", scheduleCompile);
compileButton.addEventListener("click", compileNow);
source.addEventListener("keydown", (event) => {
    if (event.key !== "Tab") {
        return;
    }

    event.preventDefault();
    const start = source.selectionStart;
    const end = source.selectionEnd;
    source.value = `${source.value.slice(0, start)}    ${source.value.slice(end)}`;
    source.selectionStart = source.selectionEnd = start + 4;
    scheduleCompile();
});
updateLineCount();
"##;

#[derive(Deserialize)]
struct CompileRequest {
    source: String,
}

#[derive(Serialize)]
struct CompileResponse {
    status: &'static str,
    output: String,
}

#[tokio::main]
async fn main() {
    let port = env::var("PORT")
        .ok()
        .and_then(|value| value.parse::<u16>().ok())
        .unwrap_or(3000);
    let addr = SocketAddr::from(([127, 0, 0, 1], port));

    let app = Router::new()
        .route("/", get(index))
        .route("/api/compile", post(compile));

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("failed to bind server socket");

    println!("nialang-online listening on http://{addr}");
    axum::serve(listener, app).await.expect("server failed");
}

async fn index() -> Html<String> {
    Html(render_page(DEFAULT_SOURCE))
}

async fn compile(Json(payload): Json<CompileRequest>) -> Json<CompileResponse> {
    let source = payload.source;
    let result = tokio::task::spawn_blocking(move || compile_source(&source))
        .await
        .unwrap_or_else(|err| CompileResponse {
            status: "error",
            output: format!("compiler task failed: {err}"),
        });

    Json(result)
}

fn compile_source(source: &str) -> CompileResponse {
    if source.trim().is_empty() {
        return CompileResponse {
            status: "error",
            output: "empty source".to_string(),
        };
    }

    match compile_to_ll(source) {
        Ok(output) => CompileResponse {
            status: "ok",
            output,
        },
        Err(output) => CompileResponse {
            status: "error",
            output,
        },
    }
}

fn render_page(source: &str) -> String {
    let initial = compile_source(source);
    let app = view! {
        <AppView
            source=source.to_string()
            output=initial.output
            is_error=initial.status == "error"
        />
    }
    .to_html();

    format!(
        r#"<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>Nia Online</title>
<style>{STYLE}</style>
</head>
<body>
{app}
<script>{CLIENT_JS}</script>
</body>
</html>"#
    )
}

#[component]
fn AppView(source: String, output: String, is_error: bool) -> impl IntoView {
    let output_class = if is_error { "output error" } else { "output" };
    let status_class = if is_error {
        "status error"
    } else {
        "status ok"
    };
    let status_text = if is_error { "Error" } else { "Compiled" };
    let line_count = source.lines().count().max(1);

    view! {
        <main class="app">
            <header class="topbar">
                <div class="brand">
                    <span class="brand-mark">"N"</span>
                    <span>"Nia Online"</span>
                </div>
                <div class="toolbar">
                    <span id="status" class=status_class>{status_text}</span>
                    <button id="compile" class="compile-button" type="button">"Compile"</button>
                </div>
            </header>
            <section class="workspace">
                <section class="pane">
                    <div class="pane-header">
                        <span class="pane-title">"Nia source"</span>
                        <span id="line-count" class="pane-meta">{line_count} " lines"</span>
                    </div>
                    <textarea
                        id="source"
                        class="editor"
                        spellcheck="false"
                        autocomplete="off"
                        autocapitalize="off"
                    >{source}</textarea>
                </section>
                <section class="pane">
                    <div class="pane-header">
                        <span class="pane-title">"LLVM IR"</span>
                        <span class="pane-meta">".ll"</span>
                    </div>
                    <pre id="output" class=output_class>{output}</pre>
                </section>
            </section>
        </main>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compile_source_returns_llvm_ir_for_valid_program() {
        let response = compile_source(DEFAULT_SOURCE);

        assert_eq!(response.status, "ok");
        assert!(
            response.output.contains("define i32 @main"),
            "{}",
            response.output
        );
    }

    #[test]
    fn compile_source_returns_error_for_invalid_program() {
        let response = compile_source("fn main() i32 { true }");

        assert_eq!(response.status, "error");
        assert!(
            response.output.contains("type error") || response.output.contains("semantic error"),
            "{}",
            response.output
        );
    }

    #[test]
    fn render_page_contains_editor_and_output_panes() {
        let page = render_page(DEFAULT_SOURCE);

        assert!(page.contains("id=\"source\""), "{page}");
        assert!(page.contains("id=\"output\""), "{page}");
        assert!(page.contains("LLVM IR"), "{page}");
    }
}
