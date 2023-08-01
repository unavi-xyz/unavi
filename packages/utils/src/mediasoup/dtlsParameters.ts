import { DtlsParameters, DtlsParameters_Role } from "@wired-protocol/types";
import { DtlsParameters as MediasoupDtlsParameters } from "mediasoup/node/lib/types";

export function fromMediasoupDtlsParameters(
  dtlsParameters: MediasoupDtlsParameters
): DtlsParameters {
  const role = fromMediasoupDtlsRole(dtlsParameters.role);
  return {
    fingerprints: dtlsParameters.fingerprints,
    role,
  };
}

function fromMediasoupDtlsRole(
  role: MediasoupDtlsParameters["role"]
): DtlsParameters_Role {
  switch (role) {
    case "auto": {
      return DtlsParameters_Role.AUTO;
    }
    case "client": {
      return DtlsParameters_Role.CLIENT;
    }
    case "server": {
      return DtlsParameters_Role.SERVER;
    }
    default: {
      throw new Error(`unknown role: ${role}`);
    }
  }
}
