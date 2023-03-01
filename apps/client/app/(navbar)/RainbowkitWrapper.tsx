"use client";

import "@rainbow-me/rainbowkit/styles.css";

import { RainbowKitProvider } from "@rainbow-me/rainbowkit";
import {
  GetSiweMessageOptions,
  RainbowKitSiweNextAuthProvider,
} from "@rainbow-me/rainbowkit-siwe-next-auth";
import { SessionProvider } from "next-auth/react";
import { WagmiConfig } from "wagmi";

import { theme } from "../../src/client/rainbow";
import { chains, wagmiClient } from "../../src/client/wagmi";

const getSiweMessageOptions: GetSiweMessageOptions = () => ({
  statement: "🔌 Sign in to the Wired",
});

interface Props {
  children: React.ReactNode;
}

export default function RainbowkitWrapper({ children }: Props) {
  return (
    <SessionProvider>
      <WagmiConfig client={wagmiClient}>
        <RainbowKitSiweNextAuthProvider getSiweMessageOptions={getSiweMessageOptions}>
          <RainbowKitProvider theme={theme} chains={chains}>
            {children}
          </RainbowKitProvider>
        </RainbowKitSiweNextAuthProvider>
      </WagmiConfig>
    </SessionProvider>
  );
}
