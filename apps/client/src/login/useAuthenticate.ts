import { getCsrfToken, signIn, useSession } from "next-auth/react";
import { useEffect } from "react";
import { useAccount, useSignMessage } from "wagmi";

import { createAuthMessage } from "./authMessage";

export function useAuthenticate() {
  const { status: accountStatus } = useAccount();
  const { signMessageAsync } = useSignMessage();

  const { status: sessionStatus } = useSession();

  // Once a wallet is connected, authenticate the user with the database
  useEffect(() => {
    if (accountStatus !== "connected" || sessionStatus !== "unauthenticated") return;

    async function authenticate() {
      // Generate a new token
      const nonce = await getCsrfToken();
      if (!nonce) throw new Error("Could not get CSRF token");

      console.log("🙅‍♂️", sessionStatus);

      // Prompt user to sign a message to verify their identity
      const message = createAuthMessage(window.location.host, nonce);
      const signature = await signMessageAsync({ message });

      signIn("credentials", { redirect: false, message, signature });
    }

    try {
      authenticate();
    } catch (e) {
      console.error(e);
    }
  }, [accountStatus, sessionStatus, signMessageAsync]);
}
