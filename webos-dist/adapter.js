function scapCallbackBridge(jsonRequestString) {
  console.log(
    "[JS Sandbox] Incoming Hardware Target Route:",
    jsonRequestString
  );
  var term = document.getElementById("logTerminal");
  if (!term) {
    term = {
      set innerHTML(val) {
        console.log(val);
      },
    };
  }

  try {
    var request = JSON.parse(jsonRequestString);
    var isRealLGTV =
      typeof window.PalmSystem !== "undefined" ||
      typeof window.PalmServiceBridge !== "undefined";

    // Standardized event dispatch closures to talk back to Rust
    function dispatchSuccess(eventType, dataPayload) {
      window.rust_process_hardware_event(
        JSON.stringify({
          req_id: request.req_id,
          event_type: eventType,
          hardware_status: "SUCCESS",
          payload: dataPayload || {},
        })
      );
    }

    function dispatchFailure() {
      window.rust_process_hardware_event(
        JSON.stringify({ req_id: request.req_id, hardware_status: "FAILED" })
      );
    }

    // SCAP Target Feature Prototype Validations
    var safeStorage =
      typeof Storage !== "undefined" &&
      Storage.prototype &&
      typeof Storage.prototype.getStorageInfo === "function";
    var safeConfig =
      typeof Configuration !== "undefined" &&
      Configuration.prototype &&
      typeof Configuration.prototype.getCurrentTime === "function";
    var safeSignage =
      typeof Signage !== "undefined" &&
      Signage.prototype &&
      typeof Signage.prototype.captureScreen === "function";
    var safePower =
      typeof Power !== "undefined" &&
      Power.prototype &&
      typeof Power.prototype.executePowerCommand === "function";

    switch (request.action) {
      case "FETCH_HARDWARE_TELEMETRY":
        if (typeof DeviceInfo !== "undefined" && isRealLGTV) {
          new DeviceInfo().getPlatformInfo(function (cb) {
            dispatchSuccess("DEVICE_INFO_CALLBACK", cb);
          }, dispatchFailure);
        } else {
          dispatchSuccess("DEVICE_INFO_CALLBACK", {
            hardwareVersion: "1",
            modelName: "SIM-SM5J",
            sdkVersion: "6.0",
            serialNumber: "SIM306INTX",
            firmwareVersion: "03.25.90",
          });
        }
        break;

      case "FETCH_NETWORK_INFO":
        if (typeof DeviceInfo !== "undefined" && isRealLGTV) {
          new DeviceInfo().getNetworkInfo(function (cb) {
            dispatchSuccess("NETWORK_INFO_CALLBACK", cb);
          }, dispatchFailure);
        } else {
          dispatchSuccess("NETWORK_INFO_CALLBACK", {
            isInternetConnectionAvailable: true,
          });
        }
        break;

      case "FETCH_STORAGE_INFO":
        if (safeStorage && isRealLGTV) {
          new Storage().getStorageInfo(function (cb) {
            dispatchSuccess("STORAGE_INFO_CALLBACK", cb);
          }, dispatchFailure);
        } else {
          dispatchSuccess("STORAGE_INFO_CALLBACK", {
            total: 4194304,
            free: 2097152,
            used: 2097152,
          });
        }
        break;

      case "EXECUTE_APP_UPGRADE":
        var targetMode =
          safeStorage && Storage.AppMode && request.options.to === "USB"
            ? Storage.AppMode.USB
            : 0;
        if (safeStorage && isRealLGTV) {
          new Storage().upgradeApplication(
            function () {
              dispatchSuccess("APP_UPGRADE_CALLBACK");
            },
            dispatchFailure,
            { to: targetMode, recovery: request.options.recovery || false }
          );
        } else {
          setTimeout(function () {
            dispatchSuccess("APP_UPGRADE_CALLBACK");
          }, 1000);
        }
        break;

      case "EXECUTE_COPY_FILE":
        if (safeStorage && isRealLGTV) {
          new Storage().copyFile(
            function () {
              dispatchSuccess("COPY_FILE_CALLBACK");
            },
            dispatchFailure,
            request.options
          );
        } else {
          setTimeout(function () {
            dispatchSuccess("COPY_FILE_CALLBACK");
          }, 500);
        }
        break;

      case "EXECUTE_FILE_EXISTS":
        if (safeStorage && isRealLGTV) {
          new Storage().exists(
            function (cb) {
              dispatchSuccess("FILE_EXISTS_CALLBACK", cb);
            },
            dispatchFailure,
            request.options
          );
        } else {
          // Simulator fallback: simulate file existing
          dispatchSuccess("FILE_EXISTS_CALLBACK", { exists: true });
        }
        break;

      case "EXECUTE_WRITE_FILE":
        if (safeStorage && isRealLGTV) {
          new Storage().writeFile(
            function () {
              dispatchSuccess("WRITE_FILE_CALLBACK");
            },
            dispatchFailure,
            request.options
          );
        } else {
          dispatchSuccess("WRITE_FILE_CALLBACK");
        }
        break;

      case "FETCH_CURRENT_TIME":
        if (safeConfig && isRealLGTV) {
          new Configuration().getCurrentTime(function (cb) {
            dispatchSuccess("TIME_INFO_CALLBACK", cb);
          }, dispatchFailure);
        } else {
          var now = new Date();
          dispatchSuccess("TIME_INFO_CALLBACK", {
            hour: now.getHours(),
            minute: now.getMinutes(),
            second: now.getSeconds(),
          });
        }
        break;

      default:
        console.warn(
          "[JS Sandbox] No execution mapping found for: ",
          request.action
        );
        break;
    }
  } catch (error) {
    console.error("[JS Sandbox] Fatal callback routing exception: ", error);
  }
}
