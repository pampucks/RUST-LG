# RUST-LG

## LG webOS Signage: Rust WebAssembly Core Bridge

This project implements a high-performance, embedded Rust-to-WebAssembly (WASM) firmware architecture driving enterprise LG digital signage hardware via the webOS Signage Core API (SCAP). This proof-of-concept establishes a secure, bi-directional command bridge, bypassing strict browser runtime sandboxing to directly manipulate physical display panels.

### 🛠️ Environment & Hardware Specification

The following setup has been fully tested, verified, and proven on real physical testbed units:

#### Verified Hardware

**Hardware Device:** LG Digital Signage Display (Standard Full HD) — **SM5J Series** (e.g., 32SM5J)

**Operating System:** **webOS Signage 6.0**

**SCAP Library Target:** Version **16** (app/javascript/cordova-cd/16/)

**Display Target:** Direct LED Panel Backlighting & System Configuration

#### Required Software & Tooling

**Rust Toolchain:** rustc / cargo **(Stable channel)**

**WASM Target Assembler:** wasm-bindgen (**v0.2.93** or newer required for module_or_path compliance)

**LG Command Line Tools:** ares-cli suite (ares-package, ares-install, ares-launch, ares-inspect)

**Execution Environment:** Internal Web Engine (Chromium WebView running over local storage protocol)

### 🏗️ Architecture & Core Logic

Driving native hardware components through an abstracted WebAssembly binary embedded within a local single-page webOS application poses two critical architectural challenges: sandboxed local storage limits and execution race conditions.

#### 1. Local Bypass Engine (File Protocol XHR Patch)

Modern webOS apps run locally under the file:// protocol. Security rules inside the built-in Chromium engine prevent the default Fetch API from loading local binaries on the disk, dropping a TypeError: Failed to fetch or ERR_ACCESS_DENIED.

To bypass this restriction, this project uses a custom multi-staged **XMLHttpRequest (XHR) ArrayBuffer loader:**

**Disk I/O Stage:** A raw local GET request retrieves the compiled .wasm file explicitly as an unparsed arraybuffer.

**Compilation Stage:** JavaScript accepts the binary byte stream directly and triggers low-level WebAssembly.compile(wasmArrayBuffer).

**Explicit Memory Mapping:** Instead of allowing the generated glue code fallback to run fetch(), we explicitly feed the engine the instantiation object parameter under the modern key module_or_path:

```
JavaScript
init({ module_or_path: compiledWasmModule })
```

#### 2. Runtime Lifecycle & Lifecycle Guard

When an operator changes system settings or drops into an OSD menu via an external LG Remote Control, webOS minimizes and suspends the running application context to background memory without killing it.

To ensure our application automatically re-asserts panel configuration metrics whenever it wakes up, a reactive lifecycle management script is wired directly to the system view metrics.

       [ Cold Boot / Warm Resume ]
                   │
                   ▼
      Is Rust Engine fully bound?
         ├── No  ──► Defer to main WASM Lifecycle Promise
         └── Yes ──► Trigger onAppStart() ──► Sync Configuration

A strict Safety Guard Check avoids race conditions when cold booting the app. It checks whether the WebAssembly compilation engine has finished mapping functions to the global environment before attempting an initial sync.

### 📋 Syntax & Bridge Protocol Specification

Communication passing through the bridge layer maps structural string commands across the JavaScript execution stack directly to the Rust binary engine.

#### JavaScript to Rust Core Payload (Inbound)

Inbound instruction payloads must follow a structured, stringified JSON schema before entry into the window.rust_process_signage_command() memory sector.

```
JSON
{
"req_id": 1,
"action": "SET_BRIGHTNESS",
"payload": {
    "target": 100
    }
}
```

#### Rust Memory Target Processing (src/lib.rs)

The core Rust engine unrolls incoming data payloads, validates data ranges against safety metrics to protect the physical panel matrix from overheating or premature burnout, and yields hardware execution instructions back to the host system context.

```
Rust
// Core logic example processing data natively out of the WASM stack
let target_brightness = cmd.payload
.and_then(|p| p.get("target").and_then(|t| t.as_u64()))
.unwrap_or(50) as u8;

// Values are extracted safely, processed, and shipped to SCAP dispatches
```

#### Rust Core to LG SCAP Confirmation (Outbound Trace)

Once received by JavaScript, the request activates the target SCAP layer (e.g., Configuration.setPictureProperty). The asynchronous hardware confirmation callback passes an event object directly back into the Rust listener function:

```
JSON
{
"req_id": 1,
"hardware_status": "APPLIED",
"detail": "Panel parameters physically updated to 100"
}
```

### 🚀 Building, Deploying, and Testing

#### Step 1: Compile the Rust Source Code

Compile and bind the Rust repository down into optimized WebAssembly assets matching our version targets:

```
Bash
wasm-pack build --target web --out-dir pkg
```

#### Step 2: Package the webOS Application

Package your source web root directory (containing index.html, adapter.js, pkg/, and your application configuration file appinfo.json):

```
Bash
ares-package /path/to/your/app/directory
```

#### Step 3: Install the Package onto the Monitor

Push the generated .ipk application file onto the configured physical display device:

```
Bash
ares-install --device TARGET_DISPLAY_NAME com.lg.app.signage.dev_1.0.0_all.ipk
```

#### Step 4: Run and Inspect Real-time Telemetry

Launch the app interface and view live console operations from the underlying hardware subsystems:

```
Bash
ares-launch com.lg.app.signage.dev --device TARGET_DISPLAY_NAME
ares-inspect --device TARGET_DISPLAY_NAME --app com.lg.app.signage.dev
```
