import { RainbowKitProvider } from "@rainbow-me/rainbowkit";
import { Session } from "next-auth";
import { SessionProvider } from "next-auth/react";
import React from "react";
import { WagmiConfig } from "wagmi";

import LoginProvider from "./auth/LoginProvider";
import RainbowAuthProvider from "./auth/RainbowAuthProvider";
import LensProvider from "./lens/LensProvider";
import { theme } from "./theme";
import { chains, wagmiClient } from "./wagmi";

interface Props {
  session: Session | null;
  children: React.ReactNode;
}

export default function ClientSideProviders({ session, children }: Props) {
  return (
    <SessionProvider session={session}>
      <LensProvider>
        <WagmiConfig client={wagmiClient}>
          <RainbowAuthProvider>
            <RainbowKitProvider theme={theme} chains={chains}>
              <LoginProvider>{children}</LoginProvider>
            </RainbowKitProvider>
          </RainbowAuthProvider>
        </WagmiConfig>
      </LensProvider>
    </SessionProvider>
  );
}
