import {
  RtpCapabilities,
  RtpCapabilities_Codec,
  RtpCapabilities_HeaderExtension,
  RtpCapabilities_HeaderExtension_Direction,
  RtpCapabilities_Kind,
} from "@wired-protocol/types";
import {
  MediaKind,
  RtcpFeedback,
  RtpCapabilities as MediasoupRtpCapabilities,
  RtpHeaderExtensionDirection,
} from "mediasoup/node/lib/types";

export function fromMediasoupRtpCapabilities(
  rtpCapabilities: MediasoupRtpCapabilities
) {
  const codecs: RtpCapabilities_Codec[] = [];
  const headerExtensions: RtpCapabilities_HeaderExtension[] = [];

  rtpCapabilities.codecs?.forEach((codec) => {
    const kind = fromMediasoupMediaKind(codec.kind);
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

  rtpCapabilities.headerExtensions?.forEach((ext) => {
    const kind = fromMediasoupMediaKind(ext.kind);
    const direction = ext.direction
      ? fromMediasoupHeaderExtensionDirection(ext.direction)
      : undefined;

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

function fromMediasoupMediaKind(kind: MediaKind): RtpCapabilities_Kind {
  switch (kind) {
    case "audio": {
      return RtpCapabilities_Kind.AUDIO;
    }

    case "video": {
      return RtpCapabilities_Kind.VIDEO;
    }
  }
}

function fromMediasoupHeaderExtensionDirection(
  direction: RtpHeaderExtensionDirection
): RtpCapabilities_HeaderExtension_Direction {
  switch (direction) {
    case "sendrecv": {
      return RtpCapabilities_HeaderExtension_Direction.SENDRECV;
    }

    case "sendonly": {
      return RtpCapabilities_HeaderExtension_Direction.SENDONLY;
    }

    case "recvonly": {
      return RtpCapabilities_HeaderExtension_Direction.RECVONLY;
    }

    case "inactive": {
      return RtpCapabilities_HeaderExtension_Direction.INACTIVE;
    }
  }
}

export function toMediasoupRtpCapabilities(
  message: RtpCapabilities
): MediasoupRtpCapabilities {
  const codecs: MediasoupRtpCapabilities["codecs"] = [];
  const headerExtensions: MediasoupRtpCapabilities["headerExtensions"] = [];

  message.codecs?.forEach((codec) => {
    const kind = toMediasoupMediaKind(codec.kind);
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
    const kind = toMediasoupMediaKind(ext.kind);
    const direction = ext.direction
      ? toMediasoupHeaderExtensionDirection(ext.direction)
      : undefined;

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

function toMediasoupMediaKind(kind: RtpCapabilities_Kind): MediaKind {
  switch (kind) {
    case RtpCapabilities_Kind.AUDIO: {
      return "audio";
    }

    case RtpCapabilities_Kind.VIDEO: {
      return "video";
    }
  }
}

function toMediasoupHeaderExtensionDirection(
  direction: RtpCapabilities_HeaderExtension_Direction
): RtpHeaderExtensionDirection {
  switch (direction) {
    case RtpCapabilities_HeaderExtension_Direction.SENDRECV: {
      return "sendrecv";
    }

    case RtpCapabilities_HeaderExtension_Direction.SENDONLY: {
      return "sendonly";
    }

    case RtpCapabilities_HeaderExtension_Direction.RECVONLY: {
      return "recvonly";
    }

    case RtpCapabilities_HeaderExtension_Direction.INACTIVE: {
      return "inactive";
    }
  }
}
