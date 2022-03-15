import React, { ReactChild, useEffect, useState } from "react";
import { IPFS, create } from "ipfs-core";

const defaultValue: { ipfs: undefined | IPFS } = { ipfs: undefined };

export const IpfsContext = React.createContext(defaultValue);

interface Props {
  children: ReactChild;
}

export function IpfsProvider({ children }: Props) {
  const [ipfs, setIpfs] = useState<IPFS>();

  useEffect(() => {
    if (ipfs) return;

    create()
      .then((res) => {
        setIpfs(res);
      })
      .catch(() => {});
  }, []);

  return (
    <IpfsContext.Provider value={{ ipfs }}>{children}</IpfsContext.Provider>
  );
}
