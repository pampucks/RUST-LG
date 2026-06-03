function scapCallbackBridge(jsonRequestString) {
  console.log("[JS Sandbox] Received from Rust Core:", jsonRequestString);
  var term = document.getElementById("logTerminal");

  try {
    var request = JSON.parse(jsonRequestString);
    var isRealLGTV =
      typeof window.PalmSystem !== "undefined" ||
      typeof window.PalmServiceBridge !== "undefined";

    switch (request.action) {
      case "SET_PICTURE_PROPERTY":
        var targetValue = request.value;
        term.innerHTML +=
          "<br>> [Rust Core Command] Action: SET_PICTURE_PROPERTY Value: " +
          targetValue;

        var options = { brightness: targetValue, backlight: targetValue };

        function successCb() {
          term.innerHTML +=
            "<br><span style='color: #00ff00;'>[Hardware Success] TV brightness updated to " +
            targetValue +
            "%</span>";
          var successResponse = {
            req_id: request.req_id,
            hardware_status: "APPLIED",
            detail: "Panel parameters physically updated to " + targetValue,
          };
          window.rust_process_hardware_event(JSON.stringify(successResponse));
        }

        function failureCb(cbObject) {
          term.innerHTML +=
            "<br><span style='color: #ff4136;'>[Hardware Error] Code [" +
            cbObject.errorCode +
            "]: " +
            cbObject.errorText +
            "</span>";
          var errorResponse = {
            req_id: request.req_id,
            hardware_status: "FAILED",
            detail:
              "Error Code [" + cbObject.errorCode + "]: " + cbObject.errorText,
          };
          window.rust_process_hardware_event(JSON.stringify(errorResponse));
        }

        if (typeof Configuration !== "undefined" && isRealLGTV) {
          var configuration = new Configuration();
          configuration.setPictureProperty(successCb, failureCb, options);
        } else {
          term.innerHTML +=
            "<br><span style='color: #ffa500;'>[Simulation Mode] Emulating device return hook...</span>";
          setTimeout(successCb, 1000);
        }
        break;

      case "FETCH_HARDWARE_TELEMETRY":
        console.log(
          "[JS Sandbox] Rust Core requested hardware metrics. Initializing SCAP DeviceInfo..."
        );

        function sendTelemetryToRust(modelName, firmwareVersion) {
          var response = {
            event_type: "DEVICE_INFO_CALLBACK",
            req_id: request.req_id,
            payload: {
              model: modelName || "Unknown LG Model",
              firmware: firmwareVersion || "Unknown FW",
            },
          };
          window.rust_process_hardware_event(JSON.stringify(response));
        }

        try {
          if (typeof DeviceInfo !== "undefined" && isRealLGTV) {
            var deviceInfo = new DeviceInfo();
            deviceInfo.getPlatformInfo(
              function successCb(cbObject) {
                sendTelemetryToRust(
                  cbObject.modelName,
                  cbObject.firmwareVersion
                );
              },
              function failureCb(cbObject) {
                console.error(
                  "[LG SCAP] Device Info Failed. Error: " + cbObject.errorText
                );
                var errorResponse = {
                  req_id: request.req_id,
                  hardware_status: "FAILED",
                };
                window.rust_process_hardware_event(
                  JSON.stringify(errorResponse)
                );
              }
            );
          } else {
            term.innerHTML +=
              "<br><span style='color: #ffa500;'>[Simulation Mode] Emulating hardware telemetry return hook...</span>";
            setTimeout(function () {
              sendTelemetryToRust("LG-32SM5J-SIMULATOR", "06.01.24");
            }, 800);
          }
        } catch (e) {
          console.error("[JS Sandbox] DeviceInfo execution failed:", e);
        }
        break;

      default:
        console.warn(
          "[JS Sandbox] Unknown action received from Rust Core:",
          request.action
        );
    }
  } catch (e) {
    console.error("[JS Sandbox] Core parsing failure:", e);
  }
}

function onAppStart() {
  // Start the Watchdog Engine: Tick Rust every 1 second to inspect flight times
  setInterval(function () {
    if (typeof window.rust_check_transaction_timeouts === "function") {
      window.rust_check_transaction_timeouts(Date.now());
    }
  }, 1000);

  // Dispatch payloads with full runtime metrics
  var brightnessPayload = {
    req_id: 1,
    action: "SET_BRIGHTNESS",
    payload: { target: 20 },
    client_timestamp_ms: Date.now(),
  };
  window.rust_process_signage_command(JSON.stringify(brightnessPayload));

  var infoPayload = {
    req_id: 2,
    action: "GET_DEVICE_INFO",
    payload: null,
    client_timestamp_ms: Date.now(),
  };
  window.rust_process_signage_command(JSON.stringify(infoPayload));
}
