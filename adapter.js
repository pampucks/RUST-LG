// adapter_lg.js
// Strictly ES5 for webOS 3.0 / 4.0 compatibility

// 1. The Bridge Function (Called by Rust)
// This function name MUST match the extern "C" declaration in Rust
window.scapCallbackBridge = function(responseJsonString) {
    var response;
    try {
        response = JSON.parse(responseJsonString);
    } catch (e) {
        console.error("Failed to parse Rust response", e);
        return;
    }
    
    // Rust has processed the logic, now we execute the physical SCAP hardware API
    if (response.status === "SUCCESS") {
        executeScapHardwareAction(response);
    }
};

// 2. The SCAP API Execution Wrapper
function executeScapHardwareAction(rustCommand) {
    // We mock the SCAP Configuration API here
    // In actual LG webOS, you would use: new Configuration().setPictureProperty(...)
    
    // Example: LG SCAP API expects callbacks for success/failure
    var options = {
        success: function(result) {
            // Hardware success! Send confirmation back to Rust
            var confirmation = {
                req_id: rustCommand.req_id,
                hardware_status: "APPLIED",
                detail: result
            };
            // Note: process_hardware_event must be exposed by your Rust Wasm build
            window.wasm_bindgen.process_hardware_event(JSON.stringify(confirmation));
        },
        failure: function(error) {
            // Hardware failure! Tell Rust to update its state machine
            var failureAlert = {
                req_id: rustCommand.req_id,
                hardware_status: "FAILED",
                detail: error.errorMessage
            };
            window.wasm_bindgen.process_hardware_event(JSON.stringify(failureAlert));
        }
    };

    // Mapping Rust Commands to actual LG SCAP APIs
    if (rustCommand.message.indexOf("Brightness") !== -1) {
        // Mocking the physical LG SCAP API call
        console.log("[LG SCAP] Adjusting Panel Brightness...");
        
        // Simulate asynchronous hardware response time (50ms)
        setTimeout(function() {
            options.success("Panel brightness physically updated");
        }, 50);
    }
}

// 3. Triggering the Chain (From the JS Frontend Event Loop)
// Example: The display wakes up, we tell Rust to calculate what needs to happen
function onAppStart() {
    var initialPayload = {
        req_id: 1,
        action: "SET_BRIGHTNESS",
        payload: { target: 80 }
    };
    
    // Send the JSON command into the Rust Brain
    window.wasm_bindgen.process_signage_command(JSON.stringify(initialPayload));
}