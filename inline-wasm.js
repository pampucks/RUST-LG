const fs = require("fs");
const path = require("path");

const wasmFile = path.join(__dirname, "pkg", "rust_lg_adapter_bg.wasm");
const outputFile = path.join(__dirname, "pkg", "wasm_bytes.js");

try {
  const buffer = fs.readFileSync(wasmFile);
  const base64 = buffer.toString("base64");
  fs.writeFileSync(outputFile, `export const wasmBase64 = "${base64}";\n`);
  console.log(
    "🚀 Success: Encoded WASM bytes generated into pkg/wasm_bytes.js"
  );
} catch (err) {
  console.error("❌ Error converting file:", err.message);
}
