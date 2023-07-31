import { createWorker } from "mediasoup";

export async function createMediasoupWorker() {
  const worker = await createWorker({
    rtcMaxPort: parseInt(process.env.RTC_MAX_PORT || "20020"),
    rtcMinPort: parseInt(process.env.RTC_MIN_PORT || "20000"),
  });

  worker.on("died", () => {
    console.error(
      "mediasoup Worker died, exiting  in 2 seconds... [pid:%d]",
      worker.pid,
    );

    setTimeout(() => process.exit(1), 2000);
  });

  const webRtcServer = await worker.createWebRtcServer({
    listenInfos: [
      {
        announcedIp: process.env.MEDIASOUP_ANNOUNCED_IP || "127.0.0.1",
        ip: process.env.MEDIASOUP_LISTEN_IP || "0.0.0.0",
        protocol: "udp",
      },
      {
        announcedIp: process.env.MEDIASOUP_ANNOUNCED_IP || "127.0.0.1",
        ip: process.env.MEDIASOUP_LISTEN_IP || "0.0.0.0",
        protocol: "tcp",
      },
    ],
  });

  const router = await worker.createRouter({
    mediaCodecs: [
      {
        channels: 2,
        clockRate: 48000,
        kind: "audio",
        mimeType: "audio/opus",
      },
    ],
  });

  return { router, webRtcServer };
}
