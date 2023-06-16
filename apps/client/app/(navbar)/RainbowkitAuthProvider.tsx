import {
  createAuthenticationAdapter,
  RainbowKitAuthenticationProvider,
} from "@rainbow-me/rainbowkit";
import { ReactNode, useMemo } from "react";
import { SiweMessage } from "siwe";

import { useAuth } from "@/src/client/AuthProvider";
import { AuthMethod } from "@/src/server/auth/types";

import { getNonce } from "../api/auth/methods/ethereum/nonce/helper";

type UnconfigurableMessageOptions = {
  address: string;
  chainId: number;
  nonce: string;
};

interface RainbowKitAuthProviderProps {
  enabled?: boolean;
  children: ReactNode;
}

/**
 * A wrapper for the RainbowKit authentication provider that uses our auth context
 */
export function RainbowKitAuthProvider({
  children,
  enabled,
}: RainbowKitAuthProviderProps) {
  const { status, login, logout } = useAuth();

  const adapter = useMemo(
    () =>
      createAuthenticationAdapter({
        createMessage: ({ address, chainId, nonce }) => {
          const unconfigurableOptions: UnconfigurableMessageOptions = {
            address,
            chainId,
            nonce,
          };

          return new SiweMessage({
            domain: window.location.host,
            statement: "Sign in to UNAVI 🌐",
            uri: window.location.origin,
            version: "1",

            ...unconfigurableOptions,
          });
        },

        getMessageBody: ({ message }) => message.prepareMessage(),

        getNonce,

        signOut: async () => {
          await logout();
        },

        verify: async ({ message, signature }) => {
          await login({
            message: JSON.stringify(message),
            method: AuthMethod.Ethereum,
            signature,
          });

          return true;
        },
      }),
    [login, logout]
  );

  return (
    <RainbowKitAuthenticationProvider
      adapter={adapter}
      enabled={enabled}
      status={status}
    >
      {children}
    </RainbowKitAuthenticationProvider>
  );
}
