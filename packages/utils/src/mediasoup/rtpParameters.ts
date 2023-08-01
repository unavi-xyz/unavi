import {
  RtpParameters,
  RtpParameters_Codec,
  RtpParameters_HeaderExtension,
} from "@wired-protocol/types";
import { RtpParameters as MediasoupRtpParameters } from "mediasoup/node/lib/types";

export function fromMediasoupRtpParameters(
  rtpParameters: MediasoupRtpParameters,
): RtpParameters {
  const codecs: RtpParameters_Codec[] = [];
  const headerExtensions: RtpParameters_HeaderExtension[] = [];

  rtpParameters.codecs?.forEach((codec) => {
    codecs.push({
      channels: codec.channels,
      clockRate: codec.clockRate,
      mimeType: codec.mimeType,
      payloadType: codec.payloadType,
    });
  });

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
  rtpParameters: RtpParameters,
): MediasoupRtpParameters {
  const codecs: MediasoupRtpParameters["codecs"] = [];
  const headerExtensions: MediasoupRtpParameters["headerExtensions"] = [];

  rtpParameters.codecs?.forEach((codec) => {
    codecs.push({
      channels: codec.channels,
      clockRate: codec.clockRate,
      mimeType: codec.mimeType,
      payloadType: codec.payloadType,
    });
  });

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
