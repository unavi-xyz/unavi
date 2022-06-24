import { ethers } from "ethers";

import { useEthersStore } from "./store";

declare let window: any;

export async function connectWallet() {
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

  useEthersStore.setState({ address, signer });
}

export function disconnectWallet() {
  useEthersStore.setState({ address: undefined, signer: undefined });
}
