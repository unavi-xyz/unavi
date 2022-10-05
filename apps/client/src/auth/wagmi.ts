import { connectorsForWallets } from "@rainbow-me/rainbowkit";
import {
  argentWallet,
  coinbaseWallet,
  injectedWallet,
  ledgerWallet,
  metaMaskWallet,
  rainbowWallet,
  walletConnectWallet,
} from "@rainbow-me/rainbowkit/wallets";
import { chain, configureChains, createClient } from "wagmi";
import { alchemyProvider } from "wagmi/providers/alchemy";
import { publicProvider } from "wagmi/providers/public";

const apiKey = process.env.ALCHEMY_ID;

export const CHAIN_IDS = [chain.polygonMumbai.id];

export const { chains, provider } = configureChains(
  [chain.polygonMumbai],
  [alchemyProvider({ apiKey }), publicProvider()]
);

const needsInjectedWalletFallback =
  typeof window !== "undefined" &&
  window.ethereum &&
  !window.ethereum.isMetaMask &&
  !window.ethereum.isCoinbaseWallet;

const connectors = connectorsForWallets([
  {
    groupName: "Popular",
    wallets: [
      metaMaskWallet({ chains }),
      rainbowWallet({ chains }),
      coinbaseWallet({ chains, appName: "The Wired" }),
    ],
  },
  {
    groupName: "More",
    wallets: [
      argentWallet({ chains }),
      ledgerWallet({ chains }),
      walletConnectWallet({ chains }),
      ...(needsInjectedWalletFallback ? [injectedWallet({ chains })] : []),
    ],
  },
]);

export const wagmiClient = createClient({ connectors, provider });
