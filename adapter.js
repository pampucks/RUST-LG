function scapCallbackBridge(jsonRequestString) {
  console.log(
    "[JS Sandbox] Incoming Hardware Target Route:",
    jsonRequestString
  );
  var term = document.getElementById("logTerminal");
  if (!term)
    term = {
      set innerHTML(val) {
        console.log(val);
      },
    };

  try {
    var request = JSON.parse(jsonRequestString);
    var isRealLGTV =
      typeof window.PalmSystem !== "undefined" ||
      typeof window.PalmServiceBridge !== "undefined";

    // Standardized event dispatch closures
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
          dispatchSuccess("FILE_EXISTS_CALLBACK", { exists: true });
        }
        break;

      case "EXECUTE_READ_FILE":
        if (safeStorage && isRealLGTV) {
          new Storage().readFile(
            function (cb) {
              dispatchSuccess("READ_FILE_CALLBACK", cb);
            },
            dispatchFailure,
            request.options
          );
        } else {
          dispatchSuccess("READ_FILE_CALLBACK", {
            data:
              request.options.encoding === "utf8"
                ? "Simulated Baseline File Payload"
                : [65, 66, 67],
          });
        }
        break;

      case "EXECUTE_LIST_FILES":
        if (safeStorage && isRealLGTV) {
          new Storage().listFiles(
            function (cb) {
              dispatchSuccess("LIST_FILES_CALLBACK", cb);
            },
            dispatchFailure,
            request.options
          );
        } else {
          dispatchSuccess("LIST_FILES_CALLBACK", {
            files: [{ name: "ad_canvas.mp4", type: "file", size: 204800 }],
          });
        }
        break;

      case "EXECUTE_STAT_FILE":
        if (safeStorage && isRealLGTV) {
          new Storage().statFile(
            function (cb) {
              dispatchSuccess("STAT_FILE_CALLBACK", cb);
            },
            dispatchFailure,
            request.options
          );
        } else {
          dispatchSuccess("STAT_FILE_CALLBACK", { size: 1024 });
        }
        break;

      case "EXECUTE_REMOVE_FILE":
        if (safeStorage && isRealLGTV) {
          new Storage().removeFile(
            function () {
              dispatchSuccess("REMOVE_FILE_CALLBACK");
            },
            dispatchFailure,
            request.options
          );
        } else {
          dispatchSuccess("REMOVE_FILE_CALLBACK");
        }
        break;

      case "EXECUTE_WRITE_FILE":
        if (safeStorage && isRealLGTV) {
          new Storage().writeFile(
            function (cb) {
              dispatchSuccess("WRITE_FILE_CALLBACK", cb);
            },
            dispatchFailure,
            request.options
          );
        } else {
          dispatchSuccess("WRITE_FILE_CALLBACK", {
            written: request.options.length || 50,
          });
        }
        break;

      case "EXECUTE_REMOVE_ALL":
        if (safeStorage && isRealLGTV) {
          new Storage().removeAll(
            function () {
              dispatchSuccess("REMOVE_ALL_CALLBACK");
            },
            dispatchFailure,
            request.options
          );
        } else {
          dispatchSuccess("REMOVE_ALL_CALLBACK");
        }
        break;

      case "EXECUTE_SCREEN_CAPTURE":
        if (safeSignage && isRealLGTV) {
          new Signage().captureScreen(
            function (cb) {
              dispatchSuccess("SCREEN_CAPTURE_CALLBACK", cb);
            },
            dispatchFailure,
            request.options
          );
        } else {
          dispatchSuccess("SCREEN_CAPTURE_CALLBACK", {
            size: 100,
            encoding: "base64",
            data: "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAAAAAA6fptVAAAACklEQVR4nGNiAAAABgABAMat3wAAAABJRU5ErkJggg==",
          });
        }
        break;

      case "FETCH_CURRENT_TIME":
        if (safeConfig && isRealLGTV) {
          new Configuration().getCurrentTime(function (cb) {
            dispatchSuccess("TIME_INFO_CALLBACK", cb);
          }, dispatchFailure);
        } else {
          var d = new Date();
          dispatchSuccess("TIME_INFO_CALLBACK", {
            year: d.getFullYear(),
            month: d.getMonth() + 1,
            day: d.getDate(),
            hour: d.getHours(),
            minute: d.getMinutes(),
            second: d.getSeconds(),
          });
        }
        break;

      case "EXECUTE_SET_SERVER":
        var modeOption =
          safeConfig &&
          Configuration.AppMode &&
          request.options.appLaunchMode === "REMOTE"
            ? Configuration.AppMode.REMOTE
            : "remote";
        var typeOption =
          safeConfig &&
          Configuration.AppType &&
          request.options.appType === "ZIP"
            ? Configuration.AppType.ZIP
            : "zip";
        if (safeConfig && isRealLGTV) {
          new Configuration().setServerProperty(
            function () {
              dispatchSuccess("SERVER_PROPERTY_CALLBACK", request.options);
            },
            dispatchFailure,
            Object.assign({}, request.options, {
              appLaunchMode: modeOption,
              appType: typeOption,
            })
          );
        } else {
          dispatchSuccess("SERVER_PROPERTY_CALLBACK", request.options);
        }
        break;

      // --- NEW HARDWARE LIFE DEPLOYMENT & POWER COMMAND METHODS (16 - 24) ---

      case "EXECUTE_APP_RESTART":
        term.innerHTML +=
          "<br>> [Rust Command] Executing SCAP restartApplication...";
        if (safeConfig && isRealLGTV) {
          new Configuration().restartApplication(function () {
            dispatchSuccess("APP_RESTART_CALLBACK");
          }, dispatchFailure);
        } else {
          term.innerHTML +=
            "<br><span style='color: #ffa500;'>[Simulation] App Restart Executed</span>";
          dispatchSuccess("APP_RESTART_CALLBACK");
        }
        break;

      case "FETCH_SERVER_PROPERTY":
        term.innerHTML +=
          "<br>> [Rust Command] Executing SCAP getServerProperty...";
        if (safeConfig && isRealLGTV) {
          new Configuration().getServerProperty(function (cb) {
            dispatchSuccess("SERVER_PROPERTY_CALLBACK", cb);
          }, dispatchFailure);
        } else {
          dispatchSuccess("SERVER_PROPERTY_CALLBACK", {
            serverIp: "10.174.243.1",
            serverPort: 80,
            secureConnection: false,
            appLaunchMode: "local",
            fqdnMode: false,
            fqdnAddr: "",
          });
        }
        break;

      case "EXECUTE_POWER_CMD":
        term.innerHTML +=
          "<br>> [Rust Command] Executing SCAP executePowerCommand...";
        var powerCmd =
          safePower &&
          Power.PowerCommand &&
          request.options.powerCommand === "REBOOT"
            ? Power.PowerCommand.REBOOT
            : "reboot";
        if (safePower && isRealLGTV) {
          new Power().executePowerCommand(
            function () {
              dispatchSuccess("POWER_CMD_CALLBACK");
            },
            dispatchFailure,
            { powerCommand: powerCmd }
          );
        } else {
          term.innerHTML +=
            "<br><span style='color: #ffa500;'>[Simulation] Power Command Issued: " +
            request.options.powerCommand +
            "</span>";
          dispatchSuccess("POWER_CMD_CALLBACK");
        }
        break;

      case "FETCH_ON_TIMER_LIST":
        if (safePower && isRealLGTV) {
          new Power().getOnTimerList(function (cb) {
            dispatchSuccess("ON_TIMER_LIST_CALLBACK", cb);
          }, dispatchFailure);
        } else {
          dispatchSuccess("ON_TIMER_LIST_CALLBACK", {
            timerList: [
              { hour: 9, minute: 0, week: 62, inputSource: "ext://hdmi:1" },
            ],
          });
        }
        break;

      case "FETCH_OFF_TIMER_LIST":
        if (safePower && isRealLGTV) {
          new Power().getOffTimerList(function (cb) {
            dispatchSuccess("OFF_TIMER_LIST_CALLBACK", cb);
          }, dispatchFailure);
        } else {
          dispatchSuccess("OFF_TIMER_LIST_CALLBACK", {
            timerList: [{ hour: 18, minute: 0, week: 62 }],
          });
        }
        break;

      case "EXECUTE_ADD_ON_TIMER":
        // Fallback calculation for native mask if the Power object isn't present during simulation
        var weekOnMask =
          safePower && Power.TimerWeek
            ? Power.TimerWeek.MONDAY + Power.TimerWeek.FRIDAY
            : 5;
        var onOpts = {
          hour: request.options.hour || 9,
          minute: request.options.minute || 0,
          week: request.options.week || weekOnMask,
          inputSource: request.options.inputSource || "ext://hdmi:1",
        };
        if (safePower && isRealLGTV) {
          new Power().addOnTimer(
            function () {
              dispatchSuccess("ADD_ON_TIMER_CALLBACK");
            },
            dispatchFailure,
            onOpts
          );
        } else {
          dispatchSuccess("ADD_ON_TIMER_CALLBACK");
        }
        break;

      case "EXECUTE_ADD_OFF_TIMER":
        var weekOffMask =
          safePower && Power.TimerWeek
            ? Power.TimerWeek.MONDAY + Power.TimerWeek.FRIDAY
            : 5;
        var offOpts = {
          hour: request.options.hour || 18,
          minute: request.options.minute || 0,
          week: request.options.week || weekOffMask,
        };
        if (safePower && isRealLGTV) {
          new Power().addOffTimer(
            function () {
              dispatchSuccess("ADD_OFF_TIMER_CALLBACK");
            },
            dispatchFailure,
            offOpts
          );
        } else {
          dispatchSuccess("ADD_OFF_TIMER_CALLBACK");
        }
        break;

      case "EXECUTE_ENABLE_ALL_ON":
        if (safePower && isRealLGTV) {
          new Power().enableAllOnTimer(
            function () {
              dispatchSuccess("ENABLE_ALL_ON_CALLBACK");
            },
            dispatchFailure,
            request.options
          );
        } else {
          dispatchSuccess("ENABLE_ALL_ON_CALLBACK");
        }
        break;

      case "EXECUTE_ENABLE_ALL_OFF":
        if (safePower && isRealLGTV) {
          new Power().enableAllOffTimer(
            function () {
              dispatchSuccess("ENABLE_ALL_OFF_CALLBACK");
            },
            dispatchFailure,
            request.options
          );
        } else {
          dispatchSuccess("ENABLE_ALL_OFF_CALLBACK");
        }
        break;

      default:
        console.warn(
          "[JS Sandbox] Unknown execution step route targeting:",
          request.action
        );
    }
  } catch (e) {
    console.error(
      "[JS Sandbox] Runtime execution fault inside bridge mapper:",
      e
    );
  }
}

// Injected application bootstrap sequence
function onAppStart() {
  setInterval(function () {
    if (typeof window.rust_check_transaction_timeouts === "function") {
      window.rust_check_transaction_timeouts(Date.now());
    }
  }, 1000);

  // Initial full hardware profile discovery sequence
  window.rust_process_signage_command(
    JSON.stringify({
      req_id: 201,
      action: "GET_DEVICE_INFO",
      client_timestamp_ms: Date.now(),
    })
  );
  window.rust_process_signage_command(
    JSON.stringify({
      req_id: 202,
      action: "GET_NETWORK_INFO",
      client_timestamp_ms: Date.now(),
    })
  );
  window.rust_process_signage_command(
    JSON.stringify({
      req_id: 203,
      action: "GET_STORAGE_INFO",
      client_timestamp_ms: Date.now(),
    })
  );
  window.rust_process_signage_command(
    JSON.stringify({
      req_id: 204,
      action: "GET_ON_TIMER_LIST",
      client_timestamp_ms: Date.now(),
    })
  );
}
