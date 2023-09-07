export type IdentityUsername = {
  type: "username";
  username: string;
};

export type IdentityAddress = {
  type: "address";
  address: string;
};

export type IdentityDid = {
  type: "did";
  did: string;
};

export type IdentityUnknown = {
  type: "unknown";
  value: string;
};

export type Identity =
  | IdentityUsername
  | IdentityAddress
  | IdentityDid
  | IdentityUnknown;

export function parseIdentity(str: string): Identity {
  // If starts with @, it's a username
  if (str.startsWith("%40")) {
    // Remove the @ from the handle
    const username = str.slice(3);
    return { type: "username", username };
  }

  // If it starts with 0x, it's an address
  if (str.startsWith("0x") && str.length === 42) {
    return { address: str, type: "address" };
  }

  // If it starts with did, it's a did
  if (str.startsWith("did")) {
    let did = str;

    // Ensure all : are encoded
    did = did.replaceAll(":", "%3A");

    if (did.startsWith("did%3Aweb%3A")) {
      // did:web does not treat : and %3A the same, so we need to handle this
      // %3A is used to specify a port, so we will assume if a number follows the
      // first %3A, it is a port and not a :
      const afterWeb = did.slice(12);

      const firstColon = afterWeb.indexOf("%3A");
      const secondColon = afterWeb.indexOf("%3A", firstColon + 1);
      const port = afterWeb.slice(firstColon + 3, secondColon);
      const isPortNumber = !isNaN(Number(port));

      // If port number, replace all but the first %3A with :
      if (isPortNumber) {
        const first = afterWeb.slice(0, firstColon);
        let second = afterWeb.slice(firstColon + 3);
        second = second.replaceAll("%3A", ":");
        did = `did:web:${first}%3A${second}`;
      } else {
        // Replace all %3A with :
        did = `did:web:${afterWeb.replaceAll("%3A", ":")}`;
      }
    }

    return { did, type: "did" };
  }

  return { type: "unknown", value: str };
}
