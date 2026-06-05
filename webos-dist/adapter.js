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
          // simulator: return empty array, semua file dianggap missing
          dispatchSuccess("LIST_FILES_CALLBACK", { files: [] });
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

      case "UPDATE_CONTENT": {
        var opts = request.options;
        var xhr = new XMLHttpRequest();
        xhr.onreadystatechange = function () {
          if (xhr.readyState == 4) {
            setTimeout(function () {
              window.rust_process_hardware_event(
                JSON.stringify({
                  req_id: request.req_id,
                  event_type: "UPDATE_CONTENT_CALLBACK",
                  hardware_status: "SUCCESS",
                  payload: {
                    status: xhr.status,
                    response_text: xhr.responseText || "",
                  },
                })
              );
            }, 0);
          }
        };
        xhr.open("POST", "http://dl.idm.digimaxsignage.com/tvapp", true);
        xhr.setRequestHeader(
          "Content-type",
          "application/x-www-form-urlencoded"
        );
        xhr.send(
          "duid=" +
            opts.kode_tv +
            "&checkdata=1&getdata=0&appver=" +
            opts.kode_tv
        );
        break;
      }

      case "JADWAL_DOWNLOAD": {
        var jOpts = request.options;
        var jSource =
          "http://dl.idm.digimaxsignage.com/app/video/" + jOpts.nama_text_file;
        var jDest = jOpts.video_folder_lg + jOpts.nama_text_file;
        if (safeStorage && isRealLGTV) {
          new Storage().copyFile(
            function () {
              setTimeout(function () {
                window.rust_process_hardware_event(
                  JSON.stringify({
                    req_id: request.req_id,
                    event_type: "JADWAL_DOWNLOAD_CALLBACK",
                    hardware_status: "SUCCESS",
                    payload: {},
                  })
                );
              }, 0);
            },
            dispatchFailure,
            { source: jSource, destination: jDest }
          );
        } else {
          // simulator: langsung success
          setTimeout(function () {
            window.rust_process_hardware_event(
              JSON.stringify({
                req_id: request.req_id,
                event_type: "JADWAL_DOWNLOAD_CALLBACK",
                hardware_status: "SUCCESS",
                payload: {},
              })
            );
          }, 500);
        }
        break;
      }

      case "UPDATE_DOWNLOAD": {
        var udOpts = request.options;
        var udXhr = new XMLHttpRequest();
        udXhr.open(
          "POST",
          "http://ul.idm.digimaxsignage.com/update-content-done",
          true
        );
        udXhr.setRequestHeader(
          "Content-type",
          "application/x-www-form-urlencoded"
        );
        udXhr.send("duid=" + udOpts.kode_tv + "&zipname=" + udOpts.nama_sesi);
        console.log("[Adapter] UPDATE_DOWNLOAD sent for", udOpts.nama_sesi);
        break;
      }

      case "EXECUTE_REMOVE_ALL": {
        if (safeStorage && isRealLGTV) {
          new Storage().removeAll(
            function () {
              setTimeout(function () {
                window.rust_process_hardware_event(
                  JSON.stringify({
                    req_id: request.req_id,
                    event_type: "REMOVE_ALL_CALLBACK",
                    hardware_status: "SUCCESS",
                    payload: {},
                  })
                );
              }, 0);
            },
            dispatchFailure,
            { device: "internal" }
          );
        } else {
          console.log("[Adapter] Simulator: removeAll skipped");
          setTimeout(function () {
            window.rust_process_hardware_event(
              JSON.stringify({
                req_id: request.req_id,
                event_type: "REMOVE_ALL_CALLBACK",
                hardware_status: "SUCCESS",
                payload: {},
              })
            );
          }, 500);
        }
        break;
      }

      case "EXECUTE_POWER_CMD": {
        var powerOpts = request.options;
        if (safePower && isRealLGTV) {
          var powerCommand =
            powerOpts.powerCommand === "REBOOT"
              ? Power.PowerCommand.REBOOT
              : Power.PowerCommand.REBOOT;
          new Power().executePowerCommand(
            function () {
              console.log("[Adapter] Power command success");
            },
            function () {
              console.log("[Adapter] Power command failed");
            },
            { powerCommand: powerCommand }
          );
        } else {
          console.log(
            "[Adapter] Simulator: power command skipped:",
            powerOpts.powerCommand
          );
        }
        break;
      }

      case "EXECUTE_SCREEN_CAPTURE": {
        var capOpts = request.options;
        if (safeSignage && isRealLGTV) {
          var captureOptions = {
            save: capOpts.save,
            thumbnail: capOpts.thumbnail,
          };
          if (capOpts.imgResolution === "HD" && Signage.ImgResolution) {
            captureOptions.imgResolution = Signage.ImgResolution.HD;
          }
          new Signage().captureScreen(
            function (csObj) {
              setTimeout(function () {
                window.rust_process_hardware_event(
                  JSON.stringify({
                    req_id: request.req_id,
                    event_type: "CAPTURE_SCREEN_CALLBACK",
                    hardware_status: "SUCCESS",
                    payload: { data: csObj.data },
                  })
                );
              }, 0);
            },
            dispatchFailure,
            captureOptions
          );
        } else {
          console.log("[Adapter] Simulator: captureScreen skipped");
          // simulator tidak kirim callback, tidak ada data
        }
        break;
      }

      case "SEND_CAPTURE": {
        var scOpts = request.options;
        var scXhr = new XMLHttpRequest();
        scXhr.onreadystatechange = function () {
          if (scXhr.readyState == 4) {
            if (scXhr.status == 200) {
              console.log("[Adapter] sendCapture | Success");
            } else {
              console.log(
                "[Adapter] sendCapture | Failed | status:",
                scXhr.status
              );
            }
          }
        };
        scXhr.open(
          "POST",
          "http://ul.idm.digimaxsignage.com/upload-screen",
          true
        );
        scXhr.setRequestHeader(
          "Content-type",
          "application/x-www-form-urlencoded"
        );
        scXhr.send(
          "duid=" +
            scOpts.kode_tv +
            "&screenimg=" +
            scOpts.capture_data +
            "&appver=" +
            scOpts.tv_app_ver
        );
        break;
      }

      case "KIRIM_LOG": {
        var klOpts = request.options;
        var logUrl = klOpts.video_folder_local + klOpts.file_log;
        var klXhr = new XMLHttpRequest();
        klXhr.open("GET", logUrl, true);
        klXhr.onreadystatechange = function () {
          if (klXhr.readyState == 4) {
            var sendLog = new XMLHttpRequest();
            sendLog.onreadystatechange = function () {
              if (sendLog.readyState == 4) {
                setTimeout(function () {
                  window.rust_process_hardware_event(
                    JSON.stringify({
                      req_id: request.req_id,
                      event_type: "KIRIM_LOG_CALLBACK",
                      hardware_status: "SUCCESS",
                      payload: { status: sendLog.status },
                    })
                  );
                }, 0);
              }
            };
            sendLog.open(
              "POST",
              "http://ul.idm.digimaxsignage.com/send-file-log",
              true
            );
            sendLog.setRequestHeader(
              "Content-type",
              "application/x-www-form-urlencoded"
            );
            sendLog.send(
              "file=" + klOpts.file_log + "&log=" + klXhr.responseText
            );
          }
        };
        klXhr.send();
        break;
      }

      case "CHECK_IPK_VERSION": {
        var ipkOpts = request.options;
        var ipkUrl = ipkOpts.url;
        var ipkXhr = new XMLHttpRequest();
        var ipkTimeout = setTimeout(function () {
          ipkXhr.abort();
          console.log("[Adapter] checkIPKVersion timed out");
        }, 15000);
        ipkXhr.onreadystatechange = function () {
          if (ipkXhr.readyState == 4) {
            clearTimeout(ipkTimeout);
            setTimeout(function () {
              window.rust_process_hardware_event(
                JSON.stringify({
                  req_id: request.req_id,
                  event_type: "CHECK_IPK_VERSION_CALLBACK",
                  hardware_status: "SUCCESS",
                  payload: {
                    status: ipkXhr.status,
                    response_text: ipkXhr.responseText || "",
                  },
                })
              );
            }, 0);
          }
        };
        ipkXhr.open("GET", ipkUrl, true);
        ipkXhr.setRequestHeader("Cache-Control", "no-cache");
        ipkXhr.setRequestHeader("Pragma", "no-cache");
        ipkXhr.send();
        break;
      }

      case "EXECUTE_SET_SERVER": {
        var ssOpts = request.options;
        if (typeof Configuration !== "undefined" && isRealLGTV) {
          var config = new Configuration();
          config.setServerProperty(
            function () {
              setTimeout(function () {
                window.rust_process_hardware_event(
                  JSON.stringify({
                    req_id: request.req_id,
                    event_type: "SET_SERVER_CALLBACK",
                    hardware_status: "SUCCESS",
                    payload: {},
                  })
                );
              }, 0);
            },
            dispatchFailure,
            ssOpts
          );
        } else {
          console.log("[Adapter] Simulator: setServerProperty skipped");
        }
        break;
      }

      case "UDP_START": {
        var udpPort = request.options.port || 9991;

        if (typeof webOS === "undefined" || !webOS.service) {
          console.log(
            "[Adapter] UDP_START: webOS.service not available (simulator), skipping"
          );
          // Di simulator, langsung jadi master tanpa election
          setTimeout(function () {
            window.rust_process_hardware_event(
              JSON.stringify({
                req_id: request.req_id,
                event_type: "UDP_START_CALLBACK",
                hardware_status: "SUCCESS",
                payload: { port: udpPort, simulated: true },
              })
            );
          }, 0);
          break;
        }

        webOS.service.request("luna://com.lg.app.signage.dev.webosservice/", {
          method: "startUDPService",
          parameters: { port: udpPort },
          onSuccess: function (res) {
            console.log("[Adapter] UDP_START onSuccess:", JSON.stringify(res));
            setTimeout(function () {
              window.rust_process_hardware_event(
                JSON.stringify({
                  req_id: request.req_id,
                  event_type: "UDP_START_CALLBACK",
                  hardware_status: "SUCCESS",
                  payload: res,
                })
              );
            }, 0);
          },
          onFailure: function (err) {
            console.error("[Adapter] UDP_START failed:", JSON.stringify(err));
            // jangan dispatchFailure, biarkan timeout watchdog handle
          },
        });
        break;
      }

      case "UDP_BROADCAST": {
        webOS.service.request("luna://com.lg.app.signage.dev.webosservice/", {
          method: "broadcastUDP",
          parameters: {
            data: request.options.data,
            port: request.options.port,
          },
          onSuccess: function () {
            setTimeout(function () {
              window.rust_process_hardware_event(
                JSON.stringify({
                  req_id: request.req_id,
                  event_type: "UDP_BROADCAST_CALLBACK",
                  hardware_status: "SUCCESS",
                  payload: {},
                })
              );
            }, 0);
          },
          onFailure: function (err) {
            console.error("[Adapter] UDP_BROADCAST failed:", err);
          },
        });
        break;
      }

      case "UDP_SUBSCRIBE": {
        webOS.service.request("luna://com.lg.app.signage.dev.webosservice/", {
          method: "subscribeUDP",
          parameters: { subscribe: true },
          onSuccess: function (res) {
            if (res.data) {
              // ada pesan masuk
              setTimeout(function () {
                window.rust_process_hardware_event(
                  JSON.stringify({
                    req_id: request.req_id,
                    event_type: "UDP_MESSAGE_RECEIVED",
                    hardware_status: "SUCCESS",
                    payload: {
                      data: res.data,
                      from_ip: res.from_ip,
                    },
                  })
                );
              }, 0);
            }
          },
          onFailure: function (err) {
            console.error("[Adapter] UDP_SUBSCRIBE failed:", err);
          },
          subscribe: true,
        });
        break;
      }

      case "UPDATE_STATUS":
        console.log(
          "[Adapter] BG tick received (not yet implemented):",
          request.action
        );
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
      // master kirim sync ke slaves
      if (typeof window.rust_send_sync_to_slaves === "function") {
        window.rust_send_sync_to_slaves();
      }
    });
    el.addEventListener("play", function () {
      var otherId = id === "videoPlayerA" ? "videoPlayerB" : "videoPlayerA";
      var other = document.getElementById(otherId);
      if (other) {
        other.style.opacity = 0;
        other.muted = true;
      }
      // tulis log tayang
      var src = el.src || "";
      var match = src.match(/([^\/]+\.mp4)$/);
      var videoName = match ? match[1] : "";
      if (videoName && typeof window.rust_write_log === "function") {
        window.rust_write_log(videoName);
      }
    });
  }
});
