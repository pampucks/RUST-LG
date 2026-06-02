// 1. Global bridge function called BY the Rust core
function scapCallbackBridge(jsonRequestString) {
  console.log(
    "[JS Adapter] Received command from Rust Core:",
    jsonRequestString
  );

  try {
    var request = JSON.parse(jsonRequestString);

    if (request.action === "SET_PICTURE_PROPERTY") {
      var targetValue = request.value;

      var options = {
        brightness: targetValue,
        backlight: targetValue,
      };

      function successCb() {
        console.log("[LG SCAP] Hardware updated successfully!");
        var successResponse = {
          req_id: request.req_id,
          hardware_status: "APPLIED",
          detail: "Panel parameters physically updated to " + targetValue,
        };
        window.wasm_bindgen.process_hardware_event(
          JSON.stringify(successResponse)
        );
      }

      function failureCb(cbObject) {
        console.error("[LG SCAP] Hardware failure reported:", cbObject);
        var errorResponse = {
          req_id: request.req_id,
          hardware_status: "FAILED",
          detail:
            "Error Code [" + cbObject.errorCode + "]: " + cbObject.errorText,
        };
        window.wasm_bindgen.process_hardware_event(
          JSON.stringify(errorResponse)
        );
      }

      // --- FIXED SAFETY CHECK ---
      // A real LG TV running webOS will ALWAYS have window.PalmSystem or window.PalmServiceBridge available.
      var isRealLGTV =
        typeof window.PalmSystem !== "undefined" ||
        typeof window.PalmServiceBridge !== "undefined";

      if (typeof Configuration !== "undefined" && isRealLGTV) {
        console.log(
          "[LG SCAP] Real LG webOS environment verified. Invoking hardware SDK..."
        );
        var configuration = new Configuration();
        configuration.setPictureProperty(successCb, failureCb, options);
      } else {
        // Safe local development simulation bypasses the broken SDK on laptops
        console.warn(
          "[LG SCAP] Local environment detected (PalmSystem missing). Simulating hardware execution..."
        );
        setTimeout(successCb, 500);
      }
    }
  } catch (e) {
    console.error("[JS Adapter] Critical bridge error parsing request:", e);
  }
}

// 2. Application Entry Point
function onAppStart() {
  console.log("[JS App] Booting application logic...");
  var initialPayload = {
    req_id: 1,
    action: "SET_BRIGHTNESS",
    payload: { target: 75 },
  };
  window.wasm_bindgen.process_signage_command(JSON.stringify(initialPayload));
}
