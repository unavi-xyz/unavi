import {
  RainbowKitAuthenticationProvider,
  createAuthenticationAdapter,
} from "@rainbow-me/rainbowkit";
import { getCsrfToken, signIn, signOut, useSession } from "next-auth/react";
import React, { ReactNode, useMemo } from "react";

import { createAuthMessage } from "./authMessage";

interface Props {
  children?: ReactNode;
  enabled?: boolean;
}

export default function RainbowAuthProvider({ children, enabled }: Props) {
  const { status } = useSession();
  const adapter = useMemo(
    () =>
      createAuthenticationAdapter({
        createMessage: ({ address, chainId, nonce }) => {
          const message = createAuthMessage(
            address,
            window.location.host,
            window.location.origin,
            chainId,
            nonce
          );

          return message;
        },

        getMessageBody: ({ message }) => message,

        getNonce: async () => {
          const nonce = await getCsrfToken();
          if (!nonce) throw new Error();
          return nonce;
        },

        signOut: async () => {
          await signOut({ redirect: false });
        },

        verify: async ({ message, signature }) => {
          const response = await signIn("credentials", {
            message: JSON.stringify(message),
            redirect: false,
            signature,
          });

          return response?.ok ?? false;
        },
      }),
    []
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
