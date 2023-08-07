import {
  RtpParameters,
  RtpParameters_Codec,
  RtpParameters_HeaderExtension,
} from "@wired-protocol/types";
import { RtpParameters as MediasoupRtpParameters } from "mediasoup/node/lib/types";

export function fromMediasoupRtpParameters(
  rtpParameters: MediasoupRtpParameters
): RtpParameters {
  const codecs: RtpParameters_Codec[] = [];
  const headerExtensions: RtpParameters_HeaderExtension[] = [];

  for (const codec of rtpParameters.codecs) {
    codecs.push({
      channels: codec.channels,
      clockRate: codec.clockRate,
      mimeType: codec.mimeType,
      payloadType: codec.payloadType,
      rtcpFeedback: codec.rtcpFeedback ?? [],
    });
  }

  rtpParameters.headerExtensions?.forEach((ext) => {
    headerExtensions.push({
      id: ext.id,
      uri: ext.uri,
    });
  });

  return {
    codecs,
    headerExtensions,
  };
}

export function toMediasoupRtpParameters(
  rtpParameters: RtpParameters
): MediasoupRtpParameters {
  const codecs: MediasoupRtpParameters["codecs"] = [];
  const headerExtensions: MediasoupRtpParameters["headerExtensions"] = [];

  for (const codec of rtpParameters.codecs) {
    codecs.push({
      channels: codec.channels,
      clockRate: codec.clockRate,
      mimeType: codec.mimeType,
      payloadType: codec.payloadType,
      rtcpFeedback: codec.rtcpFeedback,
    });
  }

  for (const ext of rtpParameters.headerExtensions) {
    headerExtensions.push({
      id: ext.id,
      uri: ext.uri,
    });
  }

  return {
    codecs,
    headerExtensions,
  };
}
