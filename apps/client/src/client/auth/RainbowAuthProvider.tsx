import {
  createAuthenticationAdapter,
  RainbowKitAuthenticationProvider,
} from "@rainbow-me/rainbowkit";
import { nanoid } from "nanoid";
import { signIn, signOut } from "next-auth/react";
import { ReactNode, useMemo } from "react";
import { useAccount } from "wagmi";

import { useChallenge } from "./useChallenge";
import { useSession } from "./useSession";

interface Props {
  children?: ReactNode;
  enabled?: boolean;
}

export default function RainbowAuthProvider({ children, enabled }: Props) {
  const { status } = useSession();
  const { address } = useAccount();
  const challenge = useChallenge(address);

  const adapter = useMemo(
    () =>
      createAuthenticationAdapter({
        createMessage: () => challenge ?? "",

        getMessageBody: ({ message }) => message,

        getNonce: async () => nanoid(),

        signOut: async () => {
          await signOut({ redirect: false });
        },

        verify: async ({ signature }) => {
          try {
            const response = await signIn("credentials", {
              address,
              signature,
              redirect: false,
            });

            return Boolean(response?.ok);
          } catch (error) {
            return false;
          }
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
