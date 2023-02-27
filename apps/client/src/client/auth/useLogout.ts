import { useRouter } from "next/navigation";
import { signOut } from "next-auth/react";

export function useLogout() {
  const router = useRouter();

  async function logout() {
    // Sign out of NextAuth
    await signOut({ redirect: false });
    // Refresh the page
    router.refresh();
  }

  return { logout };
}
