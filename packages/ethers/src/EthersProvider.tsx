import { ethers } from "ethers";
import { createContext, useState } from "react";

export const EthersContext = createContext<{
  address: string | undefined;
  signer: ethers.providers.JsonRpcSigner | undefined;
  connectWallet: () => Promise<{
    address: string | undefined;
    signer: ethers.providers.JsonRpcSigner | undefined;
  }>;
  disconnectWallet: () => void;
}>({
  address: undefined,
  signer: undefined,
  connectWallet: async () => ({
    address: undefined,
    signer: undefined,
  }),
  disconnectWallet: () => {},
});

declare let window: any;

interface Props {
  children: React.ReactNode;
}

export function EthersProvider({ children }: Props) {
  const [address, setAddress] = useState<string>();
  const [signer, setSigner] = useState<ethers.providers.JsonRpcSigner>();

  async function connectWallet() {
    try {
      //switch to correct network
      await window.ethereum.request({
        method: "wallet_addEthereumChain",
        params: [
          {
            chainId: "0x13881",
            rpcUrls: [
              "https://rpc-mumbai.maticvigil.com",
              "https://matic-mumbai.chainstacklabs.com",
              "https://rpc-mumbai.matic.today",
              "https://matic-testnet-archive-rpc.bwarelabs.com",
            ],
            chainName: "Mumbai",
            nativeCurrency: {
              name: "MATIC",
              symbol: "MATIC",
              decimals: 18,
            },
            blockExplorerUrls: ["https://mumbai.polygonscan.com/"],
          },
        ],
      });

      //sign in
      const provider = new ethers.providers.Web3Provider(window.ethereum);
      await provider.send("eth_requestAccounts", []);

      const signer = provider.getSigner();
      const address = await signer.getAddress();

      setAddress(address);
      setSigner(signer);

      return { address, signer };
    } catch (error) {
      console.error(error);

      if (address) setAddress(undefined);
      if (signer) setSigner(undefined);

      return { address: undefined, signer: undefined };
    }
  }

  function disconnectWallet() {
    if (address) setAddress(undefined);
    if (signer) setSigner(undefined);
  }

  return (
    <EthersContext.Provider
      value={{
        address,
        signer,
        connectWallet,
        disconnectWallet,
      }}
    >
      {children}
    </EthersContext.Provider>
  );
}
