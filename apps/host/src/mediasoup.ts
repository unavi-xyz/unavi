import mediasoup from "mediasoup";
import { Router } from "mediasoup/node/lib/Router";

export async function createMediasoupRouter() {
  const worker = await mediasoup.createWorker({
    rtcMinPort: parseInt(process.env.RTC_MIN_PORT || "40000"),
    rtcMaxPort: parseInt(process.env.RTC_MAX_PORT || "40100"),
  });

  worker.on("died", () => {
    console.error(
      "mediasoup Worker died, exiting  in 2 seconds... [pid:%d]",
      worker.pid
    );

    setTimeout(() => process.exit(1), 2000);
  });

  const router = await worker.createRouter({
    mediaCodecs: [
      {
        kind: "audio",
        mimeType: "audio/opus",
        clockRate: 48000,
        channels: 2,
      },
    ],
  });

  return router;
}

export async function createWebRtcTransport(router: Router) {
  const transport = await router.createWebRtcTransport({
    enableUdp: true,
    enableTcp: true,
    enableSctp: true,
    preferUdp: true,
    listenIps: [
      {
        ip: "0.0.0.0",
        announcedIp: "127.0.0.1",
      },
    ],
  });

  return {
    transport,
    params: {
      id: transport.id,
      iceParameters: transport.iceParameters,
      iceCandidates: transport.iceCandidates,
      dtlsParameters: transport.dtlsParameters,
      sctpParameters: transport.sctpParameters,
    },
  };
}
