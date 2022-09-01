import { IPFSHTTPClient, create } from "ipfs-http-client";
import { createContext, useEffect, useState } from "react";

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

export function IpfsProvider({
  url = process.env.NEXT_PUBLIC_IPFS_ENDPOINT,
  children,
}: Props) {
  const [ipfs, setIpfs] = useState<IPFSHTTPClient>();

  useEffect(() => {
    if (!url) {
      setIpfs(undefined);
      return;
    }

    const authKey = process.env.NEXT_PUBLIC_IPFS_AUTH;
    if (!authKey) throw new Error("NEXT_PUBLIC_IPFS_AUTH is not set");

    const newClient = create({
      url,
      headers: {
        Authorization: `Basic ${Buffer.from(authKey).toString("base64")}`,
      },
    });
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
