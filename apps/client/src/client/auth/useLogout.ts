import { useRouter } from "next/router";
import { signOut } from "next-auth/react";
import { useDisconnect } from "wagmi";

export function useLogout() {
  const router = useRouter();
  const { disconnectAsync } = useDisconnect();

  async function logout() {
    await Promise.all([
      // Sign out of NextAuth
      signOut({ redirect: false }),
      // Disconnect wallet
      disconnectAsync(),
    ]);

    // Reload the page
    // This is a hack to fix a bug where the RainbowKit modal is stuck open
    router.reload();
  }

  return { logout };
}
