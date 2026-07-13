use std::env;
use std::net::SocketAddr;

use axum::Router;
use axum::extract::Json;
use axum::response::Html;
use axum::routing::{get, post};
use leptos::prelude::*;
use nialang::driver::pipeline::{Backend, compile_to_ll_with};
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
    --bg: #0c0f14;
    --topbar: #111720;
    --panel: #151a22;
    --panel-strong: #1b2330;
    --editor: #0f141b;
    --border: #2b3544;
    --border-soft: #202936;
    --text: #f5f7fb;
    --muted: #9aa8ba;
    --muted-strong: #c4ccd8;
    --accent: #55d6be;
    --accent-strong: #7dd3fc;
    --accent-ink: #031317;
    --warning: #f9c76b;
    --danger: #ff8f9c;
    --danger-bg: #26161d;
    --shadow: 0 24px 80px rgb(0 0 0 / 0.35);
    --mono: "SFMono-Regular", "Cascadia Code", "Liberation Mono", Menlo, Consolas, monospace;
    --sans: Inter, ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
}

* {
    box-sizing: border-box;
}

html,
body {
    height: 100%;
    margin: 0;
    overflow: hidden;
}

body {
    background: var(--bg);
    color: var(--text);
    font-family: var(--sans);
}

.app {
    display: grid;
    grid-template-rows: auto minmax(0, 1fr);
    height: 100vh;
    overflow: hidden;
    background:
        linear-gradient(180deg, #121821 0%, #0c0f14 34%),
        var(--bg);
}

.topbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
    min-height: 64px;
    padding: 0 20px;
    border-bottom: 1px solid var(--border);
    background: color-mix(in srgb, var(--topbar) 94%, white 6%);
    box-shadow: 0 1px 0 rgb(255 255 255 / 0.04) inset;
}

.brand {
    display: inline-flex;
    align-items: center;
    gap: 12px;
    min-width: 0;
    font-weight: 700;
    letter-spacing: 0;
}

.brand-mark {
    display: grid;
    place-items: center;
    width: 34px;
    height: 34px;
    border: 1px solid #3a6570;
    border-radius: 8px;
    background: linear-gradient(145deg, #1d3340, #14212b);
    color: var(--accent);
    font-family: var(--mono);
    font-size: 16px;
    line-height: 1;
    box-shadow: 0 10px 28px rgb(0 0 0 / 0.22);
}

.toolbar {
    display: inline-flex;
    align-items: center;
    gap: 12px;
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

.quant-toggle {
    display: inline-grid;
    grid-template-columns: 18px auto;
    align-items: center;
    gap: 8px;
    min-height: 34px;
    padding: 0 10px;
    border: 1px solid var(--border);
    border-radius: 8px;
    background: #151c26;
    color: var(--muted-strong);
    font-size: 13px;
    font-weight: 650;
    cursor: pointer;
    user-select: none;
}

.quant-toggle:focus-within {
    border-color: var(--accent-strong);
    box-shadow: 0 0 0 3px rgb(125 211 252 / 0.14);
}

.quant-checkbox {
    width: 16px;
    height: 16px;
    margin: 0;
    accent-color: var(--accent);
}

.compile-button {
    min-height: 34px;
    padding: 0 16px;
    border: 1px solid #8debd4;
    border-radius: 8px;
    background: linear-gradient(180deg, #8fead7, #55d6be);
    color: var(--accent-ink);
    font: 700 14px var(--sans);
    cursor: pointer;
    box-shadow: 0 10px 28px rgb(85 214 190 / 0.16);
}

.compile-button:disabled {
    cursor: wait;
    opacity: 0.7;
}

.workspace {
    display: grid;
    grid-template-columns: minmax(320px, 0.95fr) minmax(340px, 1.05fr);
    gap: 14px;
    min-height: 0;
    padding: 14px;
    overflow: hidden;
}

.pane {
    display: grid;
    grid-template-rows: auto minmax(0, 1fr);
    min-width: 0;
    min-height: 0;
    overflow: hidden;
    border: 1px solid var(--border);
    border-radius: 8px;
    background: var(--panel);
    box-shadow: var(--shadow);
}

.pane.output-pane {
    border-color: #314055;
}

.pane-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    min-height: 44px;
    padding: 0 16px;
    border-bottom: 1px solid var(--border);
    background: var(--panel-strong);
}

.pane-title {
    overflow: hidden;
    color: #d9dee5;
    font-size: 12px;
    font-weight: 700;
    letter-spacing: 0.04em;
    text-transform: uppercase;
    text-overflow: ellipsis;
    white-space: nowrap;
}

.pane-meta {
    color: var(--muted);
    font: 12px var(--mono);
    white-space: nowrap;
}

.editor-wrap,
.output-wrap {
    min-width: 0;
    min-height: 0;
    overflow: hidden;
    background: var(--editor);
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
    background: var(--editor);
    color: var(--text);
    font: 14px/1.55 var(--mono);
    tab-size: 4;
}

.editor {
    display: block;
    resize: none;
    overflow: auto;
    caret-color: var(--accent);
    scrollbar-color: #3e5067 var(--editor);
}

.output {
    display: block;
    overflow: auto;
    overscroll-behavior: contain;
    white-space: pre;
    scrollbar-color: #3e5067 var(--editor);
}

.output.error {
    background: var(--danger-bg);
    color: #ffd6d6;
}

@media (max-width: 840px) {
    html,
    body {
        overflow: auto;
    }

    .app {
        min-height: 100vh;
        height: auto;
        overflow: visible;
    }

    .topbar {
        align-items: stretch;
        flex-direction: column;
        padding: 12px;
    }

    .toolbar {
        flex-wrap: wrap;
        justify-content: flex-start;
        width: 100%;
    }

    .workspace {
        grid-template-columns: 1fr;
        grid-template-rows: minmax(360px, 44vh) minmax(360px, 44vh);
        padding: 10px;
    }

    .pane {
        box-shadow: none;
    }
}
"#;

const CLIENT_JS: &str = r##"
const source = document.querySelector("#source");
const output = document.querySelector("#output");
const outputTitle = document.querySelector("#output-title");
const outputMeta = document.querySelector("#output-meta");
const statusLine = document.querySelector("#status");
const compileButton = document.querySelector("#compile");
const quant = document.querySelector("#quant");
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
        const quantEnabled = quant.checked;
        const response = await fetch("/api/compile", {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({ source: source.value, quant: quantEnabled }),
        });

        const payload = await response.json();
        if (currentRequest !== requestId) {
            return;
        }

        output.textContent = payload.output;
        outputTitle.textContent = quantEnabled ? "QIR" : "LLVM IR";
        outputMeta.textContent = quantEnabled ? "quant .ll" : ".ll";
        if (payload.status === "ok") {
            output.className = "output";
            statusLine.textContent = quantEnabled ? "QIR ready" : "Compiled";
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
quant.addEventListener("change", compileNow);
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
    quant: bool,
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
    let quant = payload.quant;
    let result = tokio::task::spawn_blocking(move || compile_source(&source, quant))
        .await
        .unwrap_or_else(|err| CompileResponse {
            status: "error",
            output: format!("compiler task failed: {err}"),
        });

    Json(result)
}

fn compile_source(source: &str, quant: bool) -> CompileResponse {
    if source.trim().is_empty() {
        return CompileResponse {
            status: "error",
            output: "empty source".to_string(),
        };
    }

    let backend = if quant {
        Backend::Qir
    } else {
        Backend::Default
    };
    match compile_to_ll_with(source, backend) {
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
    let initial = compile_source(source, false);
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
                    <label class="quant-toggle" for="quant">
                        <input id="quant" class="quant-checkbox" type="checkbox" />
                        <span>"Quant"</span>
                    </label>
                    <button id="compile" class="compile-button" type="button">"Compile"</button>
                </div>
            </header>
            <section class="workspace">
                <section class="pane">
                    <div class="pane-header">
                        <span class="pane-title">"Nia source"</span>
                        <span id="line-count" class="pane-meta">{line_count} " lines"</span>
                    </div>
                    <div class="editor-wrap">
                        <textarea
                            id="source"
                            class="editor"
                            spellcheck="false"
                            autocomplete="off"
                            autocapitalize="off"
                        >{source}</textarea>
                    </div>
                </section>
                <section class="pane output-pane">
                    <div class="pane-header">
                        <span id="output-title" class="pane-title">"LLVM IR"</span>
                        <span id="output-meta" class="pane-meta">".ll"</span>
                    </div>
                    <div class="output-wrap">
                        <pre id="output" class=output_class>{output}</pre>
                    </div>
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
        let response = compile_source(DEFAULT_SOURCE, false);

        assert_eq!(response.status, "ok");
        assert!(
            response.output.contains("define i32 @main"),
            "{}",
            response.output
        );
    }

    #[test]
    fn compile_source_returns_error_for_invalid_program() {
        let response = compile_source("fn main() i32 { true }", false);

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
        assert!(page.contains("id=\"quant\""), "{page}");
        assert!(page.contains("LLVM IR"), "{page}");
    }

    #[test]
    fn compile_source_can_emit_qir_view() {
        let response = compile_source(DEFAULT_SOURCE, true);

        assert_eq!(response.status, "ok");
        assert!(
            response.output.contains("generated by nialang"),
            "{}",
            response.output
        );
    }
}
