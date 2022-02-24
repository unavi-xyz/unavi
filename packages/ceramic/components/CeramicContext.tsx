import React, { ReactChild, useState } from "react";
import { ThreeIdConnect, EthereumAuthProvider } from "@3id/connect";
import { DID } from "dids";

import { ceramic, resolver } from "../constants";

declare let window: any;

const defaultValue = {
  authenticated: false,
  viewerId: "",
  connect: async () => {},
};

export const CeramicContext = React.createContext(defaultValue);

interface Props {
  children: ReactChild | ReactChild[];
}

export function CeramicProvider({ children }: Props) {
  const [authenticated, setAuthenticated] = useState(false);
  const [viewerId, setViewerId] = useState("");

  async function connect() {
    const addresses = await window.ethereum.request({
      method: "eth_requestAccounts",
    });

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
    setViewerId(did.id);
  }

  return (
    <CeramicContext.Provider value={{ authenticated, viewerId, connect }}>
      {children}
    </CeramicContext.Provider>
  );
}
