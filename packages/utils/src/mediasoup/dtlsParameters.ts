import { DtlsParameters, DtlsParameters_Role } from "@wired-protocol/types";
import { DtlsParameters as MediasoupDtlsParameters } from "mediasoup-client/lib/types";

export function fromMediasoupDtlsParameters(
  dtlsParameters: MediasoupDtlsParameters,
): DtlsParameters {
  const role = fromMediasoupDtlsRole(dtlsParameters.role);

  return {
    fingerprints: dtlsParameters.fingerprints,
    role,
  };
}

function fromMediasoupDtlsRole(
  role: MediasoupDtlsParameters["role"],
): DtlsParameters_Role {
  switch (role) {
    case "client": {
      return DtlsParameters_Role.CLIENT;
    }

    case "server": {
      return DtlsParameters_Role.SERVER;
    }

    case "auto":
    default: {
      return DtlsParameters_Role.AUTO;
    }
  }
}

export function toMediasoupDtlsParameters(
  dtlsParameters: DtlsParameters,
): MediasoupDtlsParameters {
  const role = toMediasoupDtlsRole(dtlsParameters.role);

  return {
    fingerprints: dtlsParameters.fingerprints,
    role,
  };
}

export function toMediasoupDtlsRole(
  role: DtlsParameters_Role,
): MediasoupDtlsParameters["role"] {
  switch (role) {
    case DtlsParameters_Role.AUTO: {
      return "auto";
    }

    case DtlsParameters_Role.CLIENT: {
      return "client";
    }

    case DtlsParameters_Role.SERVER: {
      return "server";
    }
  }
}
