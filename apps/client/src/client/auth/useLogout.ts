import { useRouter } from "next/navigation";
import { signOut } from "next-auth/react";

export function useLogout() {
  const router = useRouter();

  async function logout() {
    await Promise.all([
      // Sign out of NextAuth
      signOut({ redirect: false }),
    ]);

    // Reload the page
    // This is a hack to fix a bug where the RainbowKit modal is stuck open
    setTimeout(() => {
      router.refresh();
    }, 1000);
  }

  return { logout };
}
