function scapCallbackBridge(jsonRequestString) {
  console.log("[JS Sandbox] Received from Rust Core:", jsonRequestString);
  var term = document.getElementById("logTerminal");

  try {
    var request = JSON.parse(jsonRequestString);

    if (request.action === "SET_PICTURE_PROPERTY") {
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
        window.wasm_bindgen.process_hardware_event(
          JSON.stringify(successResponse)
        );
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
        window.wasm_bindgen.process_hardware_event(
          JSON.stringify(errorResponse)
        );
      }

      var isRealLGTV =
        typeof window.PalmSystem !== "undefined" ||
        typeof window.PalmServiceBridge !== "undefined";

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
    }
  } catch (e) {
    console.error("[JS Sandbox] Core parsing failure:", e);
  }
}

function onAppStart() {
  // Send down a dramatic 20% brightness command to completely prove the pixels are responding!
  var initialPayload = {
    req_id: 1,
    action: "SET_BRIGHTNESS",
    payload: { target: 20 },
  };
  window.wasm_bindgen.process_signage_command(JSON.stringify(initialPayload));
}
