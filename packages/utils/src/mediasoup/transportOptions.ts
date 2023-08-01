import {
  DtlsParameters_Role,
  IceCandidate_Protocol,
  IceCandidate_TcpType,
  IceCandidate_Type,
  TransportCreated_TransportOptions,
} from "@wired-protocol/types";
import { IceCandidate, TransportOptions } from "mediasoup-client/lib/types";

export function toMediasoupTransportOptions(
  options: TransportCreated_TransportOptions,
): TransportOptions {
  const iceCandidates: IceCandidate[] = [];

  options.iceCandidates?.forEach((candidate) => {
    let protocol: IceCandidate["protocol"];

    switch (candidate.protocol) {
      case IceCandidate_Protocol.UDP: {
        protocol = "udp";
        break;
      }

      case IceCandidate_Protocol.TCP: {
        protocol = "tcp";
        break;
      }
    }

    let type: IceCandidate["type"];

    switch (candidate.type) {
      case IceCandidate_Type.HOST: {
        type = "host";
        break;
      }

      case IceCandidate_Type.SRFLX: {
        type = "srflx";
        break;
      }

      case IceCandidate_Type.PRFLX: {
        type = "prflx";
        break;
      }

      case IceCandidate_Type.RELAY: {
        type = "relay";
        break;
      }
    }

    let tcpType: IceCandidate["tcpType"];

    switch (candidate.tcpType) {
      case IceCandidate_TcpType.ACTIVE: {
        tcpType = "active";
        break;
      }

      case IceCandidate_TcpType.PASSIVE: {
        tcpType = "passive";
        break;
      }

      case IceCandidate_TcpType.SO: {
        tcpType = "so";
        break;
      }
    }

    iceCandidates.push({
      foundation: candidate.foundation,
      ip: candidate.ip,
      port: candidate.port,
      priority: candidate.priority,
      protocol,
      tcpType,
      type,
    });
  });

  if (!options.iceParameters) {
    throw new Error("iceParameters is required");
  }

  if (!options.sctpParameters) {
    throw new Error("sctpParameters is required");
  }

  let role: TransportOptions["dtlsParameters"]["role"];

  switch (options.dtlsParameters?.role) {
    case DtlsParameters_Role.AUTO: {
      role = "auto";
      break;
    }

    case DtlsParameters_Role.CLIENT: {
      role = "client";
      break;
    }

    case DtlsParameters_Role.SERVER: {
      role = "server";
      break;
    }
  }

  const fingerprints: TransportOptions["dtlsParameters"]["fingerprints"] = [];

  options.dtlsParameters?.fingerprints.forEach((fingerprint) => {
    fingerprints.push(fingerprint);
  });

  const dtlsParameters: TransportOptions["dtlsParameters"] = {
    fingerprints,
    role,
  };

  const mediasoupOptions: TransportOptions = {
    dtlsParameters,
    iceCandidates,
    iceParameters: options.iceParameters,
    id: options.id,
    sctpParameters: {
      MIS: options.sctpParameters.mis,
      OS: options.sctpParameters.os,
      maxMessageSize: options.sctpParameters.maxMessageSize,
      port: options.sctpParameters.port,
    },
  };

  return mediasoupOptions;
}
