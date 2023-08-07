import {
  DtlsParameters,
  DtlsParameters_Role,
  IceCandidate,
  IceCandidate_Protocol,
  IceCandidate_TcpType,
  IceCandidate_Type,
  TransportCreated,
  TransportType,
} from "@wired-protocol/types";
import { Router, WebRtcServer } from "mediasoup/node/lib/types";

export async function createTransport(
  type: TransportType,
  router: Router,
  webRtcServer: WebRtcServer
) {
  const transport = await router.createWebRtcTransport({
    enableSctp: true,
    enableTcp: true,
    enableUdp: true,
    webRtcServer,
  });

  let dtlsRole: DtlsParameters_Role;

  switch (transport.dtlsParameters.role) {
    case "auto": {
      dtlsRole = DtlsParameters_Role.AUTO;
      break;
    }

    case "client": {
      dtlsRole = DtlsParameters_Role.CLIENT;
      break;
    }

    case "server": {
      dtlsRole = DtlsParameters_Role.SERVER;
      break;
    }

    default: {
      console.warn("Unknown DTLS role", transport.dtlsParameters.role);
      dtlsRole = DtlsParameters_Role.AUTO;
      break;
    }
  }

  const dtlsParameters: DtlsParameters = {
    fingerprints: transport.dtlsParameters.fingerprints,
    role: dtlsRole,
  };

  const iceCandidates: IceCandidate[] = [];

  for (const candidate of transport.iceCandidates) {
    let protocol: IceCandidate_Protocol;

    switch (candidate.protocol) {
      case "tcp": {
        protocol = IceCandidate_Protocol.TCP;
        break;
      }

      case "udp": {
        protocol = IceCandidate_Protocol.UDP;
        break;
      }

      default: {
        console.warn("Unknown ICE candidate protocol", candidate.protocol);
        protocol = IceCandidate_Protocol.UDP;
        break;
      }
    }

    let tcpType: IceCandidate_TcpType | undefined;

    if (candidate.tcpType) {
      tcpType = IceCandidate_TcpType.PASSIVE;
      break;
    }

    iceCandidates.push({
      foundation: candidate.foundation,
      ip: candidate.ip,
      port: candidate.port,
      priority: candidate.priority,
      protocol,
      tcpType,
      type: IceCandidate_Type.HOST,
    });
  }

  const message = TransportCreated.create({
    options: {
      dtlsParameters,
      iceCandidates,
      iceParameters: transport.iceParameters,
      id: transport.id,
      sctpParameters: transport.sctpParameters,
    },
    type,
  });

  return { message, transport };
}
