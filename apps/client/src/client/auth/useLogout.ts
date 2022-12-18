import { signOut } from "next-auth/react";
import { useDisconnect } from "wagmi";

export function useLogout() {
  const { disconnectAsync } = useDisconnect();

  async function logout() {
    const promises = [
      // Sign out of NextAuth
      signOut({ redirect: false }),
      // Disconnect wallet
      disconnectAsync(),
    ];

    await Promise.all(promises);
  }

  return { logout };
}
