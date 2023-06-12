import mediasoup from "mediasoup";
import { Router } from "mediasoup/node/lib/Router";
import { WebRtcServer } from "mediasoup/node/lib/WebRtcServer";

export async function createMediasoupWorker() {
  const worker = await mediasoup.createWorker({
    rtcMaxPort: parseInt(process.env.RTC_MAX_PORT || "20020"),
    rtcMinPort: parseInt(process.env.RTC_MIN_PORT || "20000"),
  });

  worker.on("died", () => {
    console.error("mediasoup Worker died, exiting  in 2 seconds... [pid:%d]", worker.pid);

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

export async function createWebRtcTransport(router: Router, webRtcServer: WebRtcServer) {
  const transport = await router.createWebRtcTransport({
    enableSctp: true,
    enableTcp: true,
    enableUdp: true,
    webRtcServer,
  });

  return {
    params: {
      dtlsParameters: transport.dtlsParameters,
      iceCandidates: transport.iceCandidates,
      iceParameters: transport.iceParameters,
      id: transport.id,
      sctpParameters: transport.sctpParameters,
    },
    transport,
  };
}
