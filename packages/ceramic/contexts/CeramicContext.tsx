import React, { ReactChild, useEffect, useState } from "react";
import { ThreeIdConnect, EthereumAuthProvider } from "@3id/connect";
import { CeramicClient } from "@ceramicnetwork/http-client";
import ThreeIdResolver from "@ceramicnetwork/3id-did-resolver";
import { TileLoader } from "@glazed/tile-loader";
import { DID } from "dids";

import { API_URL } from "../constants";

declare let window: any;

const ceramic = new CeramicClient(API_URL);
const resolver = ThreeIdResolver.getResolver(ceramic);
const loader = new TileLoader({ ceramic, cache: true });

const defaultValue = {
  ceramic,
  loader,
  authenticated: false,
  id: "",
  connect: () => {},
  disconnect: () => {},
};

export const CeramicContext = React.createContext(defaultValue);

export function CeramicProvider({ children }: { children: ReactChild }) {
  const [did, setDid] = useState(new DID({ resolver }));

  const [authenticated, setAuthenticated] = useState(false);
  const [id, setId] = useState("");

  useEffect(() => {
    ceramic.did = did;
  }, [did]);

  useEffect(() => {
    connect();
  }, []);

  async function connect() {
    const addresses = await window.ethereum.enable();
    const authProvider = new EthereumAuthProvider(
      window.ethereum,
      addresses[0]
    );

    const threeIdConnect = new ThreeIdConnect();
    await threeIdConnect.connect(authProvider);

    const provider = await threeIdConnect.getDidProvider();

    ceramic.did?.setProvider(provider);
    await ceramic.did?.authenticate();

    setAuthenticated(did.authenticated);
    setId(did.id);
  }

  function disconnect() {
    setDid(new DID({ resolver }));
    setAuthenticated(false);
    setId("");
  }

  return (
    <CeramicContext.Provider
      value={{ ceramic, loader, authenticated, id, connect, disconnect }}
    >
      {children}
    </CeramicContext.Provider>
  );
}
