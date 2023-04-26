"use client";

import "@rainbow-me/rainbowkit/styles.css";

import { lightTheme, RainbowKitProvider } from "@rainbow-me/rainbowkit";
import { WagmiConfig } from "wagmi";

import { chains, wagmiClient } from "@/src/client/wagmi";

import { RainbowKitAuthProvider } from "./RainbowkitAuthProvider";

const theme = lightTheme({
  accentColor: "#191919",
  accentColorForeground: "#ffffff",
  fontStack: "system",
  borderRadius: "large",
  overlayBlur: "small",
});

interface Props {
  children: React.ReactNode;
}

export default function RainbowkitWrapper({ children }: Props) {
  return (
    <WagmiConfig client={wagmiClient}>
      <RainbowKitAuthProvider>
        <RainbowKitProvider theme={theme} chains={chains}>
          {children}
        </RainbowKitProvider>
      </RainbowKitAuthProvider>
    </WagmiConfig>
  );
}
