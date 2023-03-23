"use client";

import "@rainbow-me/rainbowkit/styles.css";

import { lightTheme, RainbowKitProvider } from "@rainbow-me/rainbowkit";
import {
  GetSiweMessageOptions,
  RainbowKitSiweNextAuthProvider,
} from "@rainbow-me/rainbowkit-siwe-next-auth";
import { WagmiConfig } from "wagmi";

import { chains, wagmiClient } from "../../src/client/wagmi";

const theme = lightTheme({
  accentColor: "#191919",
  accentColorForeground: "#ffffff",
  fontStack: "system",
  borderRadius: "large",
  overlayBlur: "small",
});

const getSiweMessageOptions: GetSiweMessageOptions = () => ({
  statement: "🔌 Sign in to the Wired",
});

interface Props {
  children: React.ReactNode;
}

export default function RainbowkitWrapper({ children }: Props) {
  return (
    <WagmiConfig client={wagmiClient}>
      <RainbowKitSiweNextAuthProvider getSiweMessageOptions={getSiweMessageOptions}>
        <RainbowKitProvider theme={theme} chains={chains}>
          {children}
        </RainbowKitProvider>
      </RainbowKitSiweNextAuthProvider>
    </WagmiConfig>
  );
}
