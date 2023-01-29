import { RainbowKitProvider } from "@rainbow-me/rainbowkit";
import {
  GetSiweMessageOptions,
  RainbowKitSiweNextAuthProvider,
} from "@rainbow-me/rainbowkit-siwe-next-auth";
import { Session } from "next-auth";
import { SessionProvider } from "next-auth/react";
import { WagmiConfig } from "wagmi";

import { theme } from "./theme";
import { chains, wagmiClient } from "./wagmi";

const getSiweMessageOptions: GetSiweMessageOptions = () => ({
  statement: "⌛ Sign in to the Wired",
});

interface Props {
  session: Session | null;
  children: React.ReactNode;
}

export default function ClientSideProviders({ session, children }: Props) {
  return (
    <SessionProvider session={session}>
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
