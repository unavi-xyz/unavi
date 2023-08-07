"use client";

import "@rainbow-me/rainbowkit/styles.css";

import { lightTheme, RainbowKitProvider } from "@rainbow-me/rainbowkit";
import { WagmiConfig } from "wagmi";

import AuthProvider from "@/src/client/AuthProvider";
import { chains, config } from "@/src/client/wagmi";

import { RainbowKitAuthProvider } from "./RainbowkitAuthProvider";

const theme = lightTheme({
  accentColor: "#000000",
  accentColorForeground: "#ffffff",
  borderRadius: "large",
  fontStack: "system",
  overlayBlur: "small",
});

interface Props {
  children: React.ReactNode;
}

export default function RainbowkitWrapper({ children }: Props) {
  return (
    <AuthProvider>
      <WagmiConfig config={config}>
        <RainbowKitAuthProvider>
          <RainbowKitProvider theme={theme} chains={chains}>
            {children}
          </RainbowKitProvider>
        </RainbowKitAuthProvider>
      </WagmiConfig>
    </AuthProvider>
  );
}
