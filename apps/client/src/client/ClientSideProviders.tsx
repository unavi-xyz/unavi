import { RainbowKitProvider } from "@rainbow-me/rainbowkit";
import { Session } from "next-auth";
import { SessionProvider } from "next-auth/react";
import React from "react";
import { WagmiConfig } from "wagmi";

import LensProvider from "../lib/lens/LensProvider";
import LoginProvider from "./auth/LoginProvider";
import RainbowAuthProvider from "./auth/RainbowAuthProvider";
import { theme } from "./theme";
import { chains, wagmiClient } from "./wagmi";

interface Props {
  session: Session | null;
  children: React.ReactNode;
}

export default function ClientSideProviders({ session, children }: Props) {
  return (
    <WagmiConfig client={wagmiClient}>
      <SessionProvider session={session}>
        <LensProvider>
          <RainbowAuthProvider>
            <RainbowKitProvider theme={theme} chains={chains}>
              <LoginProvider>{children}</LoginProvider>
            </RainbowKitProvider>
          </RainbowAuthProvider>
        </LensProvider>
      </SessionProvider>
    </WagmiConfig>
  );
}
