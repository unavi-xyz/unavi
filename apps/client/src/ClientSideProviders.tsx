import { RainbowKitProvider } from "@rainbow-me/rainbowkit";
import { Session } from "next-auth";
import { SessionProvider } from "next-auth/react";
import React from "react";
import { WagmiConfig } from "wagmi";

import RainbowAuthProvider from "../src/login/RainbowAuthProvider";
import { theme } from "../src/login/theme";
import { chains, wagmiClient } from "../src/login/wagmi";
import LensProvider from "./lib/lens/LensProvider";
import LoginProvider from "./login/LoginProvider";

interface Props {
  session: Session;
  children: React.ReactNode;
}

export default function ClientSideProviders({ session, children }: Props) {
  return (
    <WagmiConfig client={wagmiClient}>
      <SessionProvider session={session}>
        <RainbowAuthProvider>
          <RainbowKitProvider theme={theme} chains={chains}>
            <LensProvider>
              <LoginProvider>{children}</LoginProvider>
            </LensProvider>
          </RainbowKitProvider>
        </RainbowAuthProvider>
      </SessionProvider>
    </WagmiConfig>
  );
}
