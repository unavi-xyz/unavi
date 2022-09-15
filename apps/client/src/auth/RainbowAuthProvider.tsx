import {
  createAuthenticationAdapter,
  RainbowKitAuthenticationProvider,
} from "@rainbow-me/rainbowkit";
import { signIn, signOut, useSession } from "next-auth/react";
import { ReactNode, useEffect, useMemo } from "react";
import { useAccount } from "wagmi";

import { useLens } from "../lib/lens/hooks/useLens";
import { useChallenge } from "./useChallenge";

interface Props {
  children?: ReactNode;
  enabled?: boolean;
}

export default function RainbowAuthProvider({ children, enabled }: Props) {
  const { status, data: session } = useSession();
  const { address } = useAccount();
  const challenge = useChallenge(address);
  const { setAccessToken } = useLens();

  useEffect(() => {
    if (status === "authenticated" && typeof session.accessToken === "string") {
      setAccessToken(session.accessToken);
    }
  }, [status, session, setAccessToken]);

  const adapter = useMemo(
    () =>
      createAuthenticationAdapter({
        createMessage: () => {
          const message = challenge ?? "";
          return message;
        },

        getMessageBody: ({ message }) => message,

        getNonce: async () => "dummy",

        signOut: async () => {
          await signOut({ redirect: false });
        },

        verify: async ({ signature }) => {
          const response = await signIn("credentials", {
            address,
            signature,
            redirect: false,
          });

          return response?.ok ?? false;
        },
      }),
    [challenge, address]
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
