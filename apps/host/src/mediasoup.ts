import mediasoup from "mediasoup";
import { Router } from "mediasoup/node/lib/Router";
import { WebRtcServer } from "mediasoup/node/lib/WebRtcServer";

export async function createMediasoupWorker() {
  const worker = await mediasoup.createWorker({
    rtcMinPort: parseInt(process.env.RTC_MIN_PORT || "20000"),
    rtcMaxPort: parseInt(process.env.RTC_MAX_PORT || "20040"),
  });

  worker.on("died", () => {
    console.error("mediasoup Worker died, exiting  in 2 seconds... [pid:%d]", worker.pid);

    setTimeout(() => process.exit(1), 2000);
  });

  const webRtcServer = await worker.createWebRtcServer({
    listenInfos: [
      {
        protocol: "udp",
        ip: process.env.MEDIASOUP_LISTEN_IP || "0.0.0.0",
        announcedIp: process.env.MEDIASOUP_ANNOUNCED_IP,
      },
      {
        protocol: "tcp",
        ip: process.env.MEDIASOUP_LISTEN_IP || "0.0.0.0",
        announcedIp: process.env.MEDIASOUP_ANNOUNCED_IP,
      },
    ],
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

  return { router, webRtcServer };
}

export async function createWebRtcTransport(router: Router, webRtcServer: WebRtcServer) {
  const transport = await router.createWebRtcTransport({
    enableUdp: true,
    enableTcp: true,
    enableSctp: true,
    preferTcp: true,
    webRtcServer,
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
