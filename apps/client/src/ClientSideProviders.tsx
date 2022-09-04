import { RainbowKitProvider } from "@rainbow-me/rainbowkit";
import { Session } from "next-auth";
import { SessionProvider } from "next-auth/react";
import React from "react";
import { WagmiConfig } from "wagmi";

import { IpfsProvider } from "@wired-labs/ipfs";
import { LensProvider } from "@wired-labs/lens";

import RainbowAuthProvider from "../src/login/RainbowAuthProvider";
import { theme } from "../src/login/theme";
import { chains, wagmiClient } from "../src/login/wagmi";
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
            <IpfsProvider>
              <LensProvider>
                <LoginProvider>{children}</LoginProvider>
              </LensProvider>
            </IpfsProvider>
          </RainbowKitProvider>
        </RainbowAuthProvider>
      </SessionProvider>
    </WagmiConfig>
  );
}
