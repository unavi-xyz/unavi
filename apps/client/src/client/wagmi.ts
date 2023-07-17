import { connectorsForWallets } from "@rainbow-me/rainbowkit";
import {
  coinbaseWallet,
  injectedWallet,
  metaMaskWallet,
  rainbowWallet,
  walletConnectWallet,
} from "@rainbow-me/rainbowkit/wallets";
import { Chain, Config, configureChains, createConfig } from "wagmi";
import { sepolia } from "wagmi/chains";
import { publicProvider } from "wagmi/providers/public";

import { env } from "../env.mjs";

declare global {
  interface Window {
    ethereum: any;
  }
}

const projectId = env.NEXT_PUBLIC_WALLETCONNECT_PROJECT_ID;

export const {
  chains,
  publicClient,
}: {
  chains: Chain[];
  publicClient: any;
} = configureChains([sepolia], [publicProvider()]);

const needsInjectedWalletFallback =
  typeof window !== "undefined" &&
  window.ethereum &&
  !window.ethereum.isMetaMask &&
  !window.ethereum.isCoinbaseWallet;

const connectors = connectorsForWallets([
  {
    groupName: "Popular",
    wallets: [
      metaMaskWallet({ chains, projectId }),
      rainbowWallet({ chains, projectId }),
      coinbaseWallet({ appName: "UNAVI", chains }),
      ...(needsInjectedWalletFallback ? [injectedWallet({ chains })] : []),
    ],
  },
  {
    groupName: "Other",
    wallets: [walletConnectWallet({ chains, projectId })],
  },
]);

export const config: Config = createConfig({
  autoConnect: true,
  connectors,
  publicClient,
});
