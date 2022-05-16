import { ethers } from "ethers";

import { useEthersStore } from "./store";

declare let window: any;

export async function connectWallet() {
  const provider = new ethers.providers.Web3Provider(window.ethereum);
  await provider.send("eth_requestAccounts", []);

  const signer = provider.getSigner();
  const address = await signer.getAddress();

  useEthersStore.setState({ address, signer });
}

export function disconnectWallet() {
  useEthersStore.setState({ address: undefined, signer: undefined });
}
