import { connectorsForWallets } from "@rainbow-me/rainbowkit";
import {
  coinbaseWallet,
  injectedWallet,
  metaMaskWallet,
  rainbowWallet,
  walletConnectWallet,
} from "@rainbow-me/rainbowkit/wallets";
import { ChainProviderFn, configureChains, createClient } from "wagmi";
import { arbitrumGoerli } from "wagmi/chains";
import { alchemyProvider } from "wagmi/providers/alchemy";
import { publicProvider } from "wagmi/providers/public";

import { env } from "../env/client.mjs";

const providers: ChainProviderFn<typeof arbitrumGoerli>[] = [publicProvider()];

const apiKey = env.NEXT_PUBLIC_ALCHEMY_ID;
if (apiKey) providers.unshift(alchemyProvider({ apiKey }));

export const { chains, provider } = configureChains([arbitrumGoerli], providers);

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
      ...(needsInjectedWalletFallback ? [injectedWallet({ chains })] : []),
    ],
  },
  {
    groupName: "Other",
    wallets: [walletConnectWallet({ chains })],
  },
]);

export const wagmiClient = createClient({ connectors, provider, autoConnect: true });
