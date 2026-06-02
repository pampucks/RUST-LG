function scapCallbackBridge(jsonRequestString) {
  console.log("[JS Sandbox] Received from Rust Core:", jsonRequestString);
  var term = document.getElementById("logTerminal");

  try {
    var request = JSON.parse(jsonRequestString);

    // Common environment check used across cases
    var isRealLGTV =
      typeof window.PalmSystem !== "undefined" ||
      typeof window.PalmServiceBridge !== "undefined";

    switch (request.action) {
      // 1. PHYSICAL PANEL CONTROL (SET BRIGHTNESS)
      case "SET_PICTURE_PROPERTY":
        var targetValue = request.value;
        term.innerHTML +=
          "<br>> [Rust Core Command] Action: SET_PICTURE_PROPERTY Value: " +
          targetValue;

        var options = {
          brightness: targetValue,
          backlight: targetValue,
        };

        function successCb() {
          console.log("[LG SCAP] Hardware updated successfully!");
          term.innerHTML +=
            "<br><span style='color: #00ff00;'>[Hardware Success] Physical TV brightness updated to " +
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
          console.error("[LG SCAP] Hardware failure reported:", cbObject);
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
          term.innerHTML +=
            "<br>> Native webOS container found. Dispatching hardware instruction...";
          var configuration = new Configuration();
          configuration.setPictureProperty(successCb, failureCb, options);
        } else {
          term.innerHTML +=
            "<br><span style='color: #ffa500;'>[Simulation Mode] Non-webOS container environment. Emulating device return hook...</span>";
          setTimeout(successCb, 1000);
        }
        break;

      // 2. HARDWARE TELEMETRY CORE (GET DEVICE INFO)
      case "FETCH_HARDWARE_TELEMETRY":
        console.log(
          "[JS Sandbox] Rust Core requested hardware metrics. Initializing SCAP DeviceInfo..."
        );

        // Wrap the execution callback logic up so we can use it for both real hardware and simulator
        function sendTelemetryToRust(modelName, firmwareVersion) {
          var response = {
            event_type: "DEVICE_INFO_CALLBACK",
            req_id: request.req_id, // FIXED: Changed from data to request
            payload: {
              model: modelName || "Unknown LG Model",
              firmware: firmwareVersion || "Unknown FW",
            },
          };

          if (typeof window.rust_process_hardware_event === "function") {
            window.rust_process_hardware_event(JSON.stringify(response));
          }
        }

        try {
          if (typeof DeviceInfo !== "undefined" && isRealLGTV) {
            var deviceInfo = new DeviceInfo();
            deviceInfo.getPlatformInfo(
              function successCb(cbObject) {
                console.log(
                  "[LG SCAP] System metrics acquired: " +
                    JSON.stringify(cbObject)
                );
                sendTelemetryToRust(
                  cbObject.modelName,
                  cbObject.firmwareVersion
                );
              },
              function failureCb(cbObject) {
                console.error(
                  "[LG SCAP] Device Info Failed. Error: " + cbObject.errorText
                );
              }
            );
          } else {
            term.innerHTML +=
              "<br><span style='color: #ffa500;'>[Simulation Mode] Emulating hardware telemetry return hook...</span>";
            // Simulated delay to mirror a real disk/firmware check asynchronously
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
          request.action // FIXED: Changed from data to request
        );
    }
  } catch (e) {
    console.error("[JS Sandbox] Core parsing failure:", e);
  }
}

function onAppStart() {
  // 1. Dispatch initial panel brightness parameters
  var initialPayload = {
    req_id: 1,
    action: "SET_BRIGHTNESS",
    payload: { target: 20 },
  };
  window.rust_process_signage_command(JSON.stringify(initialPayload));

  // 2. Dispatch the telemetry query immediately right after
  var infoPayload = {
    req_id: 2, // Incremented request ID
    action: "GET_DEVICE_INFO",
    payload: null,
  };
  window.rust_process_signage_command(JSON.stringify(infoPayload));
}
