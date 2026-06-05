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
      (typeof window.PalmSystem !== "undefined" ||
        typeof window.PalmServiceBridge !== "undefined") &&
      typeof DeviceInfo !== "undefined";

    // Standardized event dispatch closures to talk back to Rust
    function dispatchSuccess(eventType, dataPayload) {
      // 💡 Force the TV's synchronous execution onto the macro-task queue
      setTimeout(function () {
        window.rust_process_hardware_event(
          JSON.stringify({
            req_id: request.req_id,
            event_type: eventType,
            hardware_status: "SUCCESS",
            payload: dataPayload || {},
          })
        );
      }, 0);
    }

    function dispatchFailure() {
      // 💡 Ensure failures are safely decoupled as well
      setTimeout(function () {
        window.rust_process_hardware_event(
          JSON.stringify({ req_id: request.req_id, hardware_status: "FAILED" })
        );
      }, 0);
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
          // Simulator path — always fires in browser/emulator
          console.log(
            "[Adapter] Simulator: dispatching mock DEVICE_INFO_CALLBACK"
          );
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
            year: now.getFullYear(),
            month: now.getMonth() + 1,
            day: now.getDate(),
            hour: now.getHours(),
            minute: now.getMinutes(),
            second: now.getSeconds(),
          });
        }
        break;

      case "EXECUTE_READ_FILE":
        if (safeStorage && isRealLGTV) {
          new Storage().readFile(
            function (cb) {
              dispatchSuccess("FILE_READ_CALLBACK", cb);
            },
            dispatchFailure,
            request.options
          );
        } else {
          // Simulator fallback: return a mock playlist filename
          dispatchSuccess("FILE_READ_CALLBACK", {
            data: "sample_signage_content.mp4",
          });
        }
        break;

      case "PLAY_CONTENT": {
        var opts = request.options;
        var playerEl = document.getElementById("videoPlayer" + opts.player);
        if (!playerEl) {
          console.error("[Adapter] Player element not found: " + opts.player);
          break;
        }

        var frame = document.getElementById("videoPlayerFrame");
        if (frame) frame.style.visibility = "visible";

        if (opts.is_video) {
          playerEl.src = opts.src;
          playerEl.style.visibility = "visible";
          playerEl.style.opacity = 1;
          playerEl.load();

          // Version-aware play call
          if (opts.os_version >= 6) {
            playerEl.play();
            playerEl.muted = false;
          } else {
            setTimeout(function () {
              playerEl.play();
              playerEl.muted = false;
            }, 300);
          }
        } else {
          // Image via poster
          playerEl.poster = opts.src;
          playerEl.src = "";
          playerEl.style.visibility = "visible";
          playerEl.style.opacity = 1;

          var dur = (parseInt(opts.duration) || 10) * 1000;
          setTimeout(function () {
            window.rust_process_hardware_event(
              JSON.stringify({
                req_id: 9997,
                event_type: "CONTENT_ENDED",
                hardware_status: "SUCCESS",
                payload: {},
              })
            );
          }, dur);
        }
        break;
      }

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

// Add this to the very bottom of your webos-dist/adapter.js file
window.updateSignageUiBridge = function (elementId, action, value, extraValue) {
  var el = document.getElementById(elementId);
  if (!el) return;

  switch (action) {
    case "SET_HTML":
      el.innerHTML = value;
      break;
    case "SET_STYLE":
      el.style[value] = extraValue;
      break;
    case "ADD_CLASS":
      el.classList.add(value);
      break;
    case "REMOVE_CLASS":
      el.classList.remove(value);
      break;
  }
};

// Lightweight helper for network retries triggered by Rust
window.triggerNetworkRetryDelay = function (delayMs) {
  setTimeout(function () {
    window.rust_process_hardware_event(
      JSON.stringify({
        req_id: "REQ_RETRY_NETWORK",
        event_type: "NETWORK_RETRY_TRIGGER",
        hardware_status: "SUCCESS",
      })
    );
  }, delayMs);
};

// Wire video player ended events to Rust
["videoPlayerA", "videoPlayerB"].forEach(function (id) {
  var el = document.getElementById(id);
  if (el) {
    el.addEventListener("ended", function () {
      window.rust_process_hardware_event(
        JSON.stringify({
          req_id: 9997,
          event_type: "CONTENT_ENDED",
          hardware_status: "SUCCESS",
          payload: {},
        })
      );
    });
    el.addEventListener("play", function () {
      // Hide the other player (opacity-based dual buffer)
      var otherId = id === "videoPlayerA" ? "videoPlayerB" : "videoPlayerA";
      var other = document.getElementById(otherId);
      if (other) {
        other.style.opacity = 0;
        other.muted = true;
      }
    });
  }
});
