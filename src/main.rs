use std::env;
use std::net::SocketAddr;

use axum::Router;
use axum::extract::Json;
use axum::response::Html;
use axum::routing::{get, post};
use leptos::prelude::*;
use nialang::driver::pipeline::{Backend, compile_to_ll_with, run_qir_ll_to_string};
use serde::{Deserialize, Serialize};

const DEFAULT_SOURCE: &str = r#"fn main() i32 {
    let a: i32 = 1;
    let b = 2;
    a + b
}
"#;

#[cfg(test)]
const DEFAULT_QUANT_SOURCE: &str = r#"fn main() i32 {
    quant {
        let q = qubit();
        let r = q_measure(q);
        q_record(r);
    }
    0
}
"#;

#[cfg(test)]
const QFT4_SOURCE: &str = include_str!("../../nialang/examples/quantum/qft4.nia");

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
    --atom-bg: #282c34;
    --atom-fg: #abb2bf;
    --atom-comment: #5c6370;
    --atom-red: #e06c75;
    --atom-orange: #d19a66;
    --atom-yellow: #e5c07b;
    --atom-green: #98c379;
    --atom-cyan: #56b6c2;
    --atom-blue: #61afef;
    --atom-purple: #c678dd;
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

.run-button {
    min-height: 34px;
    padding: 0 16px;
    border: 1px solid #f4d28a;
    border-radius: 8px;
    background: linear-gradient(180deg, #f6d98f, #e5b85e);
    color: #1b1204;
    font: 700 14px var(--sans);
    cursor: pointer;
    box-shadow: 0 10px 28px rgb(229 184 94 / 0.14);
}

.compile-button:disabled,
.run-button:disabled {
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

.right-stack {
    display: grid;
    grid-template-rows: minmax(0, 1.35fr) minmax(220px, 0.65fr);
    gap: 14px;
    min-width: 0;
    min-height: 0;
    overflow: hidden;
}

.pane.run-pane {
    border-color: #3d3449;
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
    background: var(--atom-bg);
}

.editor-wrap {
    position: relative;
}

.editor-highlight,
.editor,
.output {
    width: 100%;
    height: 100%;
    min-height: 0;
    margin: 0;
    padding: 16px;
    border: 0;
    outline: 0;
    background: var(--atom-bg);
    color: var(--atom-fg);
    font: 14px/1.55 var(--mono);
    tab-size: 4;
    white-space: pre;
}

.editor-highlight,
.editor {
    position: absolute;
    inset: 0;
}

.editor-highlight {
    z-index: 1;
    overflow: hidden;
    pointer-events: none;
}

.editor {
    z-index: 2;
    display: block;
    resize: none;
    overflow: auto;
    background: transparent;
    color: transparent;
    caret-color: var(--accent);
    scrollbar-color: #3e5067 var(--atom-bg);
    -webkit-text-fill-color: transparent;
}

.editor::selection {
    background: rgb(97 175 239 / 0.32);
    color: transparent;
}

.output {
    display: block;
    overflow: auto;
    overscroll-behavior: contain;
    scrollbar-color: #3e5067 var(--atom-bg);
}

.output.error {
    background: var(--danger-bg);
    color: #ffd6d6;
}

.run-output {
    color: var(--atom-green);
}

.syntax-comment {
    color: var(--atom-comment);
    font-style: italic;
}

.syntax-keyword {
    color: var(--atom-purple);
}

.syntax-type {
    color: var(--atom-yellow);
}

.syntax-string {
    color: var(--atom-green);
}

.syntax-number {
    color: var(--atom-orange);
}

.syntax-symbol {
    color: var(--atom-blue);
}

.syntax-local {
    color: var(--atom-cyan);
}

.syntax-builtin,
.syntax-error {
    color: var(--atom-red);
}

.syntax-label {
    color: var(--atom-yellow);
}

.syntax-operator {
    color: var(--atom-cyan);
}

.syntax-punctuation {
    color: var(--atom-fg);
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
        grid-template-rows: minmax(360px, 40vh) minmax(520px, 58vh);
        padding: 10px;
    }

    .right-stack {
        grid-template-rows: minmax(300px, 1fr) minmax(220px, 0.75fr);
    }

    .pane {
        box-shadow: none;
    }
}
"#;

const CLIENT_JS: &str = r##"
const source = document.querySelector("#source");
const sourceHighlight = document.querySelector("#source-highlight");
const output = document.querySelector("#output");
const outputTitle = document.querySelector("#output-title");
const outputMeta = document.querySelector("#output-meta");
const runOutput = document.querySelector("#run-output");
const runStatus = document.querySelector("#run-status");
const statusLine = document.querySelector("#status");
const compileButton = document.querySelector("#compile");
const runButton = document.querySelector("#run-quant");
const quant = document.querySelector("#quant");
const lineCount = document.querySelector("#line-count");
let debounce = null;
let requestId = 0;
let runRequestId = 0;

const htmlEscapes = {
    "&": "&amp;",
    "<": "&lt;",
    ">": "&gt;",
    "\"": "&quot;",
    "'": "&#39;",
};

const niaKeywords = new Set([
    "as", "break", "continue", "else", "enum", "extern", "false", "fn", "for", "if", "impl",
    "let", "loop", "match", "mod", "move", "mut", "pub", "quant", "return", "struct", "true",
    "use", "while",
]);

const niaTypes = new Set([
    "bool", "char", "f32", "f64", "i8", "i16", "i32", "i64", "i128", "isize", "ptr", "qubit",
    "str", "String", "u8", "u16", "u32", "u64", "u128", "usize", "void",
]);

const niaBuiltins = new Set([
    "drop", "len", "println", "spawn",
]);

const llvmInstructions = new Set([
    "add", "addrspacecast", "alloca", "and", "ashr", "atomicrmw", "bitcast", "br", "call",
    "catchret", "catchswitch", "cleanupret", "cmpxchg", "declare", "define", "extractelement",
    "extractvalue", "fadd", "fcmp", "fdiv", "fence", "fmul", "fneg", "fpext", "fptosi",
    "fptoui", "fptrunc", "freeze", "frem", "fsub", "getelementptr", "icmp", "indirectbr",
    "insertelement", "insertvalue", "inttoptr", "invoke", "landingpad", "load", "lshr", "mul",
    "or", "phi", "ptrtoint", "resume", "ret", "sdiv", "select", "sext", "shl", "shufflevector",
    "sitofp", "srem", "store", "sub", "switch", "trunc", "udiv", "uitofp", "unreachable",
    "urem", "va_arg", "xor", "zext",
]);

const llvmTypes = new Set([
    "double", "float", "half", "label", "metadata", "ptr", "token", "void", "x86_fp80",
]);

const llvmAttrs = new Set([
    "acq_rel", "acquire", "align", "alwaysinline", "attributes", "cold", "dso_local", "exact",
    "fast", "global", "inbounds", "internal", "local_unnamed_addr", "monotonic", "noundef",
    "nounwind", "nsw", "nuw", "private", "release", "seq_cst", "source_filename", "target",
    "unnamed_addr", "weak",
]);

function updateLineCount() {
    const lines = source.value.length === 0 ? 1 : source.value.split("\n").length;
    lineCount.textContent = `${lines} lines`;
}

function escapeHtml(value) {
    return value.replace(/[&<>"']/g, (ch) => htmlEscapes[ch]);
}

function token(className, value) {
    return `<span class="${className}">${escapeHtml(value)}</span>`;
}

function finishHighlight(sourceText, html) {
    if (html.length === 0) {
        return " ";
    }
    return sourceText.endsWith("\n") ? `${html} ` : html;
}

function isIdentStart(ch) {
    return /[A-Za-z_]/.test(ch);
}

function isIdent(ch) {
    return /[A-Za-z0-9_]/.test(ch);
}

function isLlvmIdentStart(ch) {
    return /[A-Za-z_.$]/.test(ch);
}

function isLlvmIdent(ch) {
    return /[A-Za-z0-9_.$-]/.test(ch);
}

function readQuoted(line, quoteIndex, quoteChar) {
    let index = quoteIndex + 1;
    let escaped = false;
    while (index < line.length) {
        const ch = line[index];
        index += 1;
        if (escaped) {
            escaped = false;
        } else if (ch === "\\") {
            escaped = true;
        } else if (ch === quoteChar) {
            break;
        }
    }
    return index;
}

function highlightNiaLine(line) {
    let html = "";
    let index = 0;

    while (index < line.length) {
        if (line.startsWith("//", index)) {
            html += token("syntax-comment", line.slice(index));
            break;
        }

        const ch = line[index];
        if (ch === "\"") {
            const end = readQuoted(line, index, "\"");
            html += token("syntax-string", line.slice(index, end));
            index = end;
            continue;
        }

        if (ch === "'") {
            const end = readQuoted(line, index, "'");
            html += token("syntax-string", line.slice(index, end));
            index = end;
            continue;
        }

        const number = line.slice(index).match(/^-?(?:0x[0-9a-fA-F_]+|\d[\d_]*(?:\.\d[\d_]*)?)/);
        if (number) {
            html += token("syntax-number", number[0]);
            index += number[0].length;
            continue;
        }

        if (isIdentStart(ch)) {
            let end = index + 1;
            while (end < line.length && isIdent(line[end])) {
                end += 1;
            }
            const word = line.slice(index, end);
            if (niaKeywords.has(word)) {
                html += token("syntax-keyword", word);
            } else if (niaTypes.has(word)) {
                html += token("syntax-type", word);
            } else if (niaBuiltins.has(word)) {
                html += token("syntax-builtin", word);
            } else {
                html += escapeHtml(word);
            }
            index = end;
            continue;
        }

        if ("+-*/%=!<>&|^~:?".includes(ch)) {
            html += token("syntax-operator", ch);
        } else if ("(){}[];,.@".includes(ch)) {
            html += token("syntax-punctuation", ch);
        } else {
            html += escapeHtml(ch);
        }
        index += 1;
    }

    return html;
}

function highlightNia(text) {
    return finishHighlight(text, text.split("\n").map(highlightNiaLine).join("\n"));
}

function highlightLlvmLine(line) {
    let html = "";
    let index = 0;

    while (index < line.length) {
        const ch = line[index];
        if (ch === ";") {
            html += token("syntax-comment", line.slice(index));
            break;
        }

        if (ch === "\"" || (ch === "c" && line[index + 1] === "\"")) {
            const quoteIndex = ch === "c" ? index + 1 : index;
            const end = readQuoted(line, quoteIndex, "\"");
            html += token("syntax-string", line.slice(index, end));
            index = end;
            continue;
        }

        if (ch === "@" || ch === "%" || ch === "!" || ch === "#") {
            let end = index + 1;
            while (end < line.length && isLlvmIdent(line[end])) {
                end += 1;
            }
            const className = ch === "%" ? "syntax-local" : "syntax-symbol";
            html += token(className, line.slice(index, end));
            index = end;
            continue;
        }

        const number = line.slice(index).match(/^-?(?:0x[0-9a-fA-F]+|\d+(?:\.\d+)?(?:e[+-]?\d+)?)/i);
        if (number) {
            html += token("syntax-number", number[0]);
            index += number[0].length;
            continue;
        }

        if (isLlvmIdentStart(ch)) {
            let end = index + 1;
            while (end < line.length && isLlvmIdent(line[end])) {
                end += 1;
            }
            const word = line.slice(index, end);
            if (line[end] === ":") {
                html += token("syntax-label", `${word}:`);
                index = end + 1;
                continue;
            }
            if (llvmInstructions.has(word)) {
                html += token("syntax-keyword", word);
            } else if (llvmTypes.has(word) || /^i\d+$/.test(word) || /^v\d+i\d+$/.test(word)) {
                html += token("syntax-type", word);
            } else if (llvmAttrs.has(word)) {
                html += token("syntax-builtin", word);
            } else if (word === "false" || word === "null" || word === "poison" || word === "true" || word === "undef" || word === "zeroinitializer") {
                html += token("syntax-number", word);
            } else {
                html += escapeHtml(word);
            }
            index = end;
            continue;
        }

        if ("+-*/%=!<>&|^~:?".includes(ch)) {
            html += token("syntax-operator", ch);
        } else if ("(){}[];,.x".includes(ch)) {
            html += token("syntax-punctuation", ch);
        } else {
            html += escapeHtml(ch);
        }
        index += 1;
    }

    return html;
}

function highlightLlvm(text) {
    return finishHighlight(text, text.split("\n").map(highlightLlvmLine).join("\n"));
}

function highlightDiagnostic(text) {
    const highlighted = escapeHtml(text).replace(
        /(type error|parse error|semantic error|backend error|lex error|error|failed|panic)/gi,
        '<span class="syntax-error">$1</span>',
    );
    return finishHighlight(text, highlighted);
}

function syncSourceHighlight() {
    sourceHighlight.scrollTop = source.scrollTop;
    sourceHighlight.scrollLeft = source.scrollLeft;
}

function renderSourceHighlight() {
    sourceHighlight.innerHTML = highlightNia(source.value);
    syncSourceHighlight();
}

function renderHighlighted(target, text, mode) {
    target.innerHTML = mode === "diagnostic" ? highlightDiagnostic(text) : highlightLlvm(text);
}

function renderOutputHighlight(text, mode) {
    renderHighlighted(output, text, mode);
}

function renderRunHighlight(text, mode) {
    renderHighlighted(runOutput, text, mode);
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

        outputTitle.textContent = quantEnabled ? "QIR" : "LLVM IR";
        outputMeta.textContent = quantEnabled ? "quant .ll" : ".ll";
        if (payload.status === "ok") {
            output.className = "output";
            renderOutputHighlight(payload.output, "llvm");
            statusLine.textContent = quantEnabled ? "QIR ready" : "Compiled";
            statusLine.className = "status ok";
        } else {
            output.className = "output error";
            renderOutputHighlight(payload.output, "diagnostic");
            statusLine.textContent = "Error";
            statusLine.className = "status error";
        }
    } catch (error) {
        if (currentRequest !== requestId) {
            return;
        }

        output.className = "output error";
        renderOutputHighlight(`request failed: ${error}`, "diagnostic");
        statusLine.textContent = "Request failed";
        statusLine.className = "status error";
    } finally {
        if (currentRequest === requestId) {
            compileButton.disabled = false;
        }
    }
}

async function runQuantNow() {
    const currentRequest = ++runRequestId;
    quant.checked = true;
    runButton.disabled = true;
    runStatus.textContent = "Running";
    runStatus.className = "pane-meta";

    try {
        const response = await fetch("/api/run-quant", {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({ source: source.value }),
        });

        const payload = await response.json();
        if (currentRequest !== runRequestId) {
            return;
        }

        if (payload.status === "ok") {
            runOutput.className = "output run-output";
            renderRunHighlight(payload.output, "llvm");
            runStatus.textContent = "Completed";
        } else {
            runOutput.className = "output error";
            renderRunHighlight(payload.output, "diagnostic");
            runStatus.textContent = "Error";
        }
    } catch (error) {
        if (currentRequest !== runRequestId) {
            return;
        }

        runOutput.className = "output error";
        renderRunHighlight(`request failed: ${error}`, "diagnostic");
        runStatus.textContent = "Request failed";
    } finally {
        if (currentRequest === runRequestId) {
            runButton.disabled = false;
        }
    }
}

function scheduleCompile() {
    clearTimeout(debounce);
    renderSourceHighlight();
    updateLineCount();
    statusLine.textContent = "Edited";
    statusLine.className = "status";
    runStatus.textContent = "Stale";
    debounce = setTimeout(compileNow, 450);
}

source.addEventListener("input", scheduleCompile);
source.addEventListener("scroll", syncSourceHighlight);
quant.addEventListener("change", compileNow);
compileButton.addEventListener("click", compileNow);
runButton.addEventListener("click", () => {
    compileNow();
    runQuantNow();
});
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
renderSourceHighlight();
renderOutputHighlight(output.textContent, output.classList.contains("error") ? "diagnostic" : "llvm");
renderRunHighlight(runOutput.textContent, runOutput.classList.contains("error") ? "diagnostic" : "llvm");
updateLineCount();
"##;

#[derive(Deserialize)]
struct CompileRequest {
    source: String,
    quant: bool,
}

#[derive(Deserialize)]
struct RunQuantRequest {
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
        .route("/api/compile", post(compile))
        .route("/api/run-quant", post(run_quant));

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

async fn run_quant(Json(payload): Json<RunQuantRequest>) -> Json<CompileResponse> {
    let source = payload.source;
    let result = tokio::task::spawn_blocking(move || run_quant_source(&source))
        .await
        .unwrap_or_else(|err| CompileResponse {
            status: "error",
            output: format!("quantum runner task failed: {err}"),
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

fn run_quant_source(source: &str) -> CompileResponse {
    if source.trim().is_empty() {
        return CompileResponse {
            status: "error",
            output: "empty source".to_string(),
        };
    }

    match compile_to_ll_with(source, Backend::Qir).and_then(|ll| run_qir_ll_to_string(&ll)) {
        Ok(output) => CompileResponse {
            status: "ok",
            output: if output.trim().is_empty() {
                "program completed without textual output".to_string()
            } else {
                output
            },
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
                    <button id="run-quant" class="run-button" type="button">"Run Quant"</button>
                </div>
            </header>
            <section class="workspace">
                <section class="pane">
                    <div class="pane-header">
                        <span class="pane-title">"Nia source"</span>
                        <span id="line-count" class="pane-meta">{line_count} " lines"</span>
                    </div>
                    <div class="editor-wrap">
                        <pre id="source-highlight" class="editor-highlight" aria-hidden="true"></pre>
                        <textarea
                            id="source"
                            class="editor"
                            spellcheck="false"
                            autocomplete="off"
                            autocapitalize="off"
                            wrap="off"
                        >{source}</textarea>
                    </div>
                </section>
                <div class="right-stack">
                    <section class="pane output-pane">
                        <div class="pane-header">
                            <span id="output-title" class="pane-title">"LLVM IR"</span>
                            <span id="output-meta" class="pane-meta">".ll"</span>
                        </div>
                        <div class="output-wrap">
                            <pre id="output" class=output_class>{output}</pre>
                        </div>
                    </section>
                    <section class="pane run-pane">
                        <div class="pane-header">
                            <span class="pane-title">"Run output"</span>
                            <span id="run-status" class="pane-meta">"Idle"</span>
                        </div>
                        <div class="output-wrap">
                            <pre id="run-output" class="output run-output">"Click Run Quant to execute the QIR program."</pre>
                        </div>
                    </section>
                </div>
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
        assert!(page.contains("id=\"run-output\""), "{page}");
        assert!(page.contains("id=\"run-quant\""), "{page}");
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

    #[test]
    fn run_quant_source_returns_error_for_invalid_program() {
        let response = run_quant_source("fn main() i32 { true }");

        assert_eq!(response.status, "error");
        assert!(
            response.output.contains("type error") || response.output.contains("semantic error"),
            "{}",
            response.output
        );
    }

    #[test]
    fn run_quant_source_executes_quantum_program() {
        let response = run_quant_source(DEFAULT_QUANT_SOURCE);

        assert_eq!(response.status, "ok", "{}", response.output);
        assert!(!response.output.trim().is_empty(), "{}", response.output);
    }

    #[test]
    fn run_quant_source_reports_qft4_resources() {
        let response = run_quant_source(QFT4_SOURCE);

        assert_eq!(response.status, "ok", "{}", response.output);
        assert!(
            response.output.contains("METADATA\trequired_num_qubits\t4"),
            "{}",
            response.output
        );
        assert!(
            response
                .output
                .contains("METADATA\trequired_num_results\t4"),
            "{}",
            response.output
        );
        assert!(
            response.output.contains("OUTPUT\tRESULT"),
            "{}",
            response.output
        );
    }
}
