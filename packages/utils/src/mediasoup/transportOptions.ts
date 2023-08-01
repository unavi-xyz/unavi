import {
  IceCandidate_Protocol,
  IceCandidate_TcpType,
  IceCandidate_Type,
  TransportCreated_TransportOptions,
} from "@wired-protocol/types";
import { IceCandidate, TransportOptions } from "mediasoup-client/lib/types";

import { toMediasoupDtlsRole } from "./dtlsParameters";

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

  const role = options.dtlsParameters?.role
    ? toMediasoupDtlsRole(options.dtlsParameters.role)
    : undefined;

  const fingerprints: TransportOptions["dtlsParameters"]["fingerprints"] = [];

  options.dtlsParameters?.fingerprints.forEach((fingerprint) => {
    fingerprints.push(fingerprint);
  });

  const dtlsParameters: TransportOptions["dtlsParameters"] = {
    fingerprints,
    role,
  };

  const sctpParameters = options.sctpParameters
    ? {
      MIS: options.sctpParameters.mis,
      OS: options.sctpParameters.os,
      maxMessageSize: options.sctpParameters.maxMessageSize,
      port: options.sctpParameters.port,
    }
    : undefined;

  if (!options.iceParameters) {
    throw new Error("Missing iceParameters");
  }

  const mediasoupOptions: TransportOptions = {
    dtlsParameters,
    iceCandidates,
    iceParameters: options.iceParameters,
    id: options.id,
    sctpParameters,
  };

  return mediasoupOptions;
}
