import React, { ReactChild, useState } from "react";
import { ThreeIdConnect, EthereumAuthProvider } from "@3id/connect";
import { CeramicClient } from "@ceramicnetwork/http-client";
import ThreeIdResolver from "@ceramicnetwork/3id-did-resolver";
import { TileLoader } from "@glazed/tile-loader";
import { DID } from "dids";

import { API_URL } from "../constants";

declare let window: any;

export const ceramicRead = new CeramicClient(API_URL);
export const ceramic = new CeramicClient(API_URL);
const resolver = ThreeIdResolver.getResolver(ceramic);
export const loader = new TileLoader({ ceramic, cache: true });

const defaultValue = {
  authenticated: false,
  id: undefined as string | undefined,
  connect: () => {},
  disconnect: () => {},
};

export const CeramicContext = React.createContext(defaultValue);

export function CeramicProvider({ children }: { children: ReactChild }) {
  const [authenticated, setAuthenticated] = useState(false);
  const [id, setId] = useState<string>();

  async function connect() {
    const addresses = await window.ethereum.enable();
    const authProvider = new EthereumAuthProvider(
      window.ethereum,
      addresses[0]
    );

    const threeIdConnect = new ThreeIdConnect();
    await threeIdConnect.connect(authProvider);

    const provider = await threeIdConnect.getDidProvider();

    const did = new DID({ resolver });
    did.setProvider(provider);
    await did.authenticate();
    ceramic.did = did;

    setAuthenticated(did.authenticated);
    setId(did.id);
  }

  function disconnect() {
    setAuthenticated(false);
    setId("");
  }

  return (
    <CeramicContext.Provider
      value={{
        authenticated,
        id,
        connect,
        disconnect,
      }}
    >
      {children}
    </CeramicContext.Provider>
  );
}
