import { IPFSHTTPClient, create } from "ipfs-http-client";
import { createContext, useEffect, useState } from "react";

import { DEFAULT_ENDPOINT } from "./constants";

export const IpfsContext = createContext<{
  ipfs: IPFSHTTPClient | undefined;
  loadFromIpfs: (hash: string) => Promise<string | undefined>;
  uploadFileToIpfs: (file: File) => Promise<string | undefined>;
  uploadStringToIpfs: (str: string) => Promise<string | undefined>;
}>({
  ipfs: undefined,
  loadFromIpfs: async () => "",
  uploadFileToIpfs: async () => "",
  uploadStringToIpfs: async () => "",
});

interface Props {
  url?: string;
  children: React.ReactNode;
}

export function IpfsProvider({ url = DEFAULT_ENDPOINT, children }: Props) {
  const [ipfs, setIpfs] = useState<IPFSHTTPClient>();

  useEffect(() => {
    if (!url) {
      setIpfs(undefined);
      return;
    }

    const newClient = create({ url });
    setIpfs(newClient);
  }, [url]);

  async function loadFromIpfs(hash: string) {
    if (!ipfs) return;

    try {
      const res = await ipfs.cat(hash);

      const files: Uint8Array[] = [];
      for await (const file of res) {
        files.push(file);
      }

      const blob = new Blob(files);
      const url = URL.createObjectURL(blob);

      return url;
    } catch (error) {
      console.error(error);
    }
  }

  async function uploadFileToIpfs(file: File) {
    if (!ipfs) return;

    try {
      const buffer = await new Response(file).arrayBuffer();
      const { cid } = await ipfs.add(buffer);
      const hash = cid.toString();

      return `ipfs://${hash}`;
    } catch (error) {
      console.error(error);
    }
  }

  async function uploadStringToIpfs(str: string) {
    if (!ipfs) return;

    try {
      const buffer = new TextEncoder().encode(str);
      const { cid } = await ipfs.add(buffer);
      const hash = cid.toString();

      return `ipfs://${hash}`;
    } catch (error) {
      console.error(error);
    }
  }

  return (
    <IpfsContext.Provider
      value={{
        ipfs,
        loadFromIpfs,
        uploadFileToIpfs,
        uploadStringToIpfs,
      }}
    >
      {children}
    </IpfsContext.Provider>
  );
}
