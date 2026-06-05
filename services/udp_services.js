"use strict";

const dgram = require("dgram");
const Service = require("webos-service");

const service = new Service("com.lg.app.signage.webosservice");

let udpSocket = null;
let isListening = false;
let broadcastPort = 9991; // default, bisa dioverride
let onReceiveCallback = null;

service.register("testDgram", function (message) {
  try {
    var dgram = require("dgram");
    message.respond({
      returnValue: true,
      available: true,
      version: process.version,
    });
  } catch (e) {
    message.respond({
      returnValue: true,
      available: false,
      error: e.message,
      version: process.version,
    });
  }
});

// ─── startUDPService ───────────────────────────────────────────────
service.register("startUDPService", function (message) {
  const params = message.payload;
  broadcastPort = params.port || 9991;

  if (udpSocket) {
    try {
      udpSocket.close();
    } catch (e) {}
    udpSocket = null;
    isListening = false;
  }

  udpSocket = dgram.createSocket({ type: "udp4", reuseAddr: true });

  udpSocket.on("error", function (err) {
    console.error("[UDP] Socket error:", err.message);
    isListening = false;
    message.respond({ returnValue: false, error: err.message });
  });

  udpSocket.on("listening", function () {
    udpSocket.setBroadcast(true);
    isListening = true;
    console.log("[UDP] Listening on port", broadcastPort);
    message.respond({ returnValue: true, port: broadcastPort });
  });

  udpSocket.on("message", function (msg, rinfo) {
    var received = msg.toString();
    console.log("[UDP] Received:", received, "from", rinfo.address);

    // kirim ke semua subscriber yang sedang listen
    service.activityManager.create("udp_receive", function () {});

    // respond ke semua pending onReceiveUDP subscribers
    if (onReceiveCallback) {
      try {
        onReceiveCallback({
          returnValue: true,
          data: received,
          from_ip: rinfo.address,
        });
      } catch (e) {
        console.error("[UDP] onReceiveCallback error:", e);
      }
    }
  });

  udpSocket.bind(broadcastPort);
});

// ─── broadcastUDP ──────────────────────────────────────────────────
service.register("broadcastUDP", function (message) {
  const params = message.payload;
  const data = params.data || "";
  const port = params.port || broadcastPort;

  if (!udpSocket || !isListening) {
    message.respond({ returnValue: false, error: "UDP socket not ready" });
    return;
  }

  const buf = Buffer.from(data);
  udpSocket.send(buf, 0, buf.length, port, "255.255.255.255", function (err) {
    if (err) {
      console.error("[UDP] Broadcast error:", err.message);
      message.respond({ returnValue: false, error: err.message });
    } else {
      console.log("[UDP] Broadcast sent:", data);
      message.respond({ returnValue: true });
    }
  });
});

// ─── sendUDP (unicast ke IP spesifik) ─────────────────────────────
service.register("sendUDP", function (message) {
  const params = message.payload;
  const data = params.data || "";
  const targetIP = params.target_ip;
  const port = params.port || broadcastPort;

  if (!udpSocket || !isListening) {
    message.respond({ returnValue: false, error: "UDP socket not ready" });
    return;
  }

  const buf = Buffer.from(data);
  udpSocket.send(buf, 0, buf.length, port, targetIP, function (err) {
    if (err) {
      message.respond({ returnValue: false, error: err.message });
    } else {
      message.respond({ returnValue: true });
    }
  });
});

// ─── subscribeUDP (subscribe untuk terima pesan masuk) ────────────
service.register("subscribeUDP", function (message) {
  if (!message.isSubscription) {
    message.respond({ returnValue: false, error: "Must be a subscription" });
    return;
  }

  console.log("[UDP] New subscriber registered");
  onReceiveCallback = function (payload) {
    message.respond(payload);
  };

  // konfirmasi subscription berhasil
  message.respond({ returnValue: true, subscribed: true });
});

// ─── stopUDPService ────────────────────────────────────────────────
service.register("stopUDPService", function (message) {
  if (udpSocket) {
    try {
      udpSocket.close();
    } catch (e) {}
    udpSocket = null;
    isListening = false;
    onReceiveCallback = null;
  }
  message.respond({ returnValue: true });
});
