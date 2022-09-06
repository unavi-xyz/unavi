import { connectorsForWallets, wallet } from "@rainbow-me/rainbowkit";
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
      wallet.metaMask({ chains }),
      wallet.rainbow({ chains }),
      wallet.coinbase({ chains, appName: "The Wired" }),
    ],
  },
  {
    groupName: "More",
    wallets: [
      wallet.argent({ chains }),
      wallet.ledger({ chains }),
      wallet.walletConnect({ chains }),
      ...(needsInjectedWalletFallback ? [wallet.injected({ chains })] : []),
    ],
  },
]);

export const wagmiClient: any = createClient({
  connectors,
  provider,
});
