import { ThreeIdConnect, EthereumAuthProvider } from "@3id/connect";
import { DID } from "dids";
import { atom, useAtom } from "jotai";
import { ceramic, resolver } from "../client";

declare let window: any;

const authenticatedAtom = atom(false);
const viewerIdAtom = atom("");

export function useAuth() {
  const [authenticated, setAuthenticated] = useAtom(authenticatedAtom);
  const [viewerId, setViewerId] = useAtom(viewerIdAtom);

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

  return { authenticated, viewerId, connect };
}
