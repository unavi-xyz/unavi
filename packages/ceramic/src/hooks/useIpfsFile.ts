import { useContext } from "react";
import { useQuery } from "react-query";

import { IpfsContext } from "../components/IpfsProvider";
import { loadFromIpfs } from "../ipfs";

export function useIpfsFile(cid: string) {
  const stripped = cid?.replace("ipfs://", "");

  const { ipfs } = useContext(IpfsContext);

  async function fetcher() {
    if (!stripped || !ipfs) return;
    const file = await loadFromIpfs(ipfs, stripped);
    return file;
  }

  const { data } = useQuery(`${ipfs && "ipfs"}-${stripped}`, fetcher);

  return data;
}
