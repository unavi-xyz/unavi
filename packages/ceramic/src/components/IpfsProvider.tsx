import React, { ReactChild, useEffect, useState } from "react";
import { IPFS, create } from "ipfs-core";
import { create as createHttp } from "ipfs-http-client";

const useHttp = true;

const peers = [
  "/dnsaddr/ipfs.infura.io/tcp/5001/https",
  "/dnsaddr/bootstrap.libp2p.io/p2p/QmNnooDu7bfjPFoTZYxMNLWUQJyrVwtbZg5gBMjTezGAJN",
  "/dnsaddr/bootstrap.libp2p.io/p2p/QmQCU2EcMqAqQPR2i9bChDtGNJchTbq5TbXJJ16u19uLTa",
  "/dnsaddr/bootstrap.libp2p.io/p2p/QmbLHAnMoJPWSCR5Zhtx6BHJX9KiKNN6tpvbUcqanj75Nb",
  "/dnsaddr/bootstrap.libp2p.io/p2p/QmcZf59bWwK5XFi76CZX8cbJ4BhTzzA3gU1ZjYZcYW3dwt",
];

const defaultValue: { ipfs: undefined | IPFS } = { ipfs: undefined };
export const IpfsContext = React.createContext(defaultValue);

interface Props {
  children: ReactChild;
}

export function IpfsProvider({ children }: Props) {
  const [ipfs, setIpfs] = useState<IPFS>();

  useEffect(() => {
    if (ipfs) return;

    if (useHttp) {
      //@ts-ignore
      const res = createHttp("https://ipfs.infura.io:5001");
      setIpfs(res);
    } else {
      create({
        config: { Bootstrap: peers },
      })
        .then((res) => {
          setIpfs(res);
        })
        .catch(() => {});
    }
  }, []);

  return (
    <IpfsContext.Provider value={{ ipfs }}>{children}</IpfsContext.Provider>
  );
}
