import {
  RouterRtpCapabilities,
  RtpCapabilities,
  RtpCapabilities_Codec,
  RtpCapabilities_HeaderExtension,
  RtpCapabilities_HeaderExtension_Direction,
  RtpCapabilities_Kind,
} from "@wired-protocol/types";
import {
  MediaKind,
  Router,
  RtcpFeedback,
  RtpCapabilities as MediasoupRtpCapabilities,
  RtpHeaderExtensionDirection,
} from "mediasoup/node/lib/types";

export function createRouterRtpCapabilities(router: Router) {
  const codecs: RtpCapabilities_Codec[] = [];
  const headerExtensions: RtpCapabilities_HeaderExtension[] = [];

  router.rtpCapabilities.codecs?.forEach((codec) => {
    let kind: RtpCapabilities_Kind;

    switch (codec.kind) {
      case "audio": {
        kind = RtpCapabilities_Kind.AUDIO;
        break;
      }

      case "video": {
        kind = RtpCapabilities_Kind.VIDEO;
        break;
      }
    }

    const rtcpFeedback: RtpCapabilities_Codec["rtcpFeedback"] = [];

    codec.rtcpFeedback?.forEach((fb) => {
      rtcpFeedback.push(fb);
    });

    codecs.push({
      channels: codec.channels,
      clockRate: codec.clockRate,
      kind,
      mimeType: codec.mimeType,
      rtcpFeedback,
    });
  });

  router.rtpCapabilities.headerExtensions?.forEach((ext) => {
    let kind: RtpCapabilities_Kind;

    switch (ext.kind) {
      case "audio": {
        kind = RtpCapabilities_Kind.AUDIO;
        break;
      }

      case "video": {
        kind = RtpCapabilities_Kind.VIDEO;
        break;
      }
    }

    let direction: RtpCapabilities_HeaderExtension_Direction;

    switch (ext.direction) {
      case "sendrecv": {
        direction = RtpCapabilities_HeaderExtension_Direction.SENDRECV;
        break;
      }

      case "sendonly": {
        direction = RtpCapabilities_HeaderExtension_Direction.SENDONLY;
        break;
      }

      case "recvonly": {
        direction = RtpCapabilities_HeaderExtension_Direction.RECVONLY;
        break;
      }

      case "inactive": {
        direction = RtpCapabilities_HeaderExtension_Direction.INACTIVE;
        break;
      }

      default: {
        throw new Error(`Unknown direction "${ext.direction}"`);
      }
    }

    headerExtensions.push({
      direction,
      kind,
      preferredEncrypt: ext.preferredEncrypt,
      preferredId: ext.preferredId,
      uri: ext.uri,
    });
  });

  const message = RouterRtpCapabilities.create({
    rtpCapabilities: {
      codecs,
      headerExtensions,
    },
  });

  return message;
}

export function createMediasoupRtpCapabilities(
  message: RtpCapabilities,
): MediasoupRtpCapabilities {
  const codecs: MediasoupRtpCapabilities["codecs"] = [];
  const headerExtensions: MediasoupRtpCapabilities["headerExtensions"] = [];

  message.codecs?.forEach((codec) => {
    let kind: MediaKind;

    switch (codec.kind) {
      case RtpCapabilities_Kind.AUDIO: {
        kind = "audio";
        break;
      }

      case RtpCapabilities_Kind.VIDEO: {
        kind = "video";
        break;
      }
    }

    const rtcpFeedback: RtcpFeedback[] = [];

    codec.rtcpFeedback?.forEach((fb) => {
      rtcpFeedback.push(fb);
    });

    codecs.push({
      channels: codec.channels,
      clockRate: codec.clockRate,
      kind,
      mimeType: codec.mimeType,
      rtcpFeedback,
    });
  });

  message.headerExtensions?.forEach((ext) => {
    let kind: MediaKind;

    switch (ext.kind) {
      case RtpCapabilities_Kind.AUDIO: {
        kind = "audio";
        break;
      }

      case RtpCapabilities_Kind.VIDEO: {
        kind = "video";
        break;
      }
    }

    let direction: RtpHeaderExtensionDirection;

    switch (ext.direction) {
      case RtpCapabilities_HeaderExtension_Direction.SENDRECV: {
        direction = "sendrecv";
        break;
      }

      case RtpCapabilities_HeaderExtension_Direction.SENDONLY: {
        direction = "sendonly";
        break;
      }

      case RtpCapabilities_HeaderExtension_Direction.RECVONLY: {
        direction = "recvonly";
        break;
      }

      case RtpCapabilities_HeaderExtension_Direction.INACTIVE: {
        direction = "inactive";
        break;
      }

      default: {
        throw new Error(`Unknown direction "${ext.direction}"`);
      }
    }

    headerExtensions.push({
      direction,
      kind,
      preferredEncrypt: ext.preferredEncrypt,
      preferredId: ext.preferredId,
      uri: ext.uri,
    });
  });

  return {
    codecs,
    headerExtensions,
  };
}
