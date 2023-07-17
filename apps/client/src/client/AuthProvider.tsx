import { AuthenticationStatus } from "@rainbow-me/rainbowkit";
import { User } from "lucia-auth";
import { useRouter } from "next/navigation";
import {
  Context,
  createContext,
  useCallback,
  useContext,
  useEffect,
  useTransition,
} from "react";

import { LoginResponse } from "@/app/api/auth/login/types";
import { getAuthStatus } from "@/app/api/auth/status/helper";
import { AuthData } from "@/src/server/auth/types";

import { useAuthStore } from "./authStore";

export type AuthContextValue = {
  status: AuthenticationStatus;
  loading: boolean; // Sseparate loading state for transitions
  user: User | null;
  login: (args: AuthData) => Promise<void>;
  logout: () => Promise<void>;
};

export const AuthContext: Context<AuthContextValue> =
  createContext<AuthContextValue>({
    loading: false,
    login: async () => {},
    logout: async () => {},
    status: useAuthStore.getState().status,
    user: useAuthStore.getState().user,
  });

interface Props {
  children: React.ReactNode;
}

/**
 * A context provider for authentication
 */
export default function AuthProvider({ children }: Props) {
  const status = useAuthStore((state) => state.status);
  const setStatus = useAuthStore((state) => state.setStatus);
  const user = useAuthStore((state) => state.user);
  const setUser = useAuthStore((state) => state.setUser);

  const [loading, startTransition] = useTransition();

  const router = useRouter();

  const login = useCallback(
    async (args: AuthData) => {
      if (status === "authenticated" || status === "loading") return;

      setStatus("loading");

      try {
        const res = await fetch("/api/auth/login", {
          body: JSON.stringify(args),
          headers: { "Content-Type": "application/json" },
          method: "POST",
        });
        if (!res.ok) throw new Error("Login failed");

        const { user } = (await res.json()) as LoginResponse;

        // Refresh the page
        startTransition(() => router.refresh());

        setStatus("authenticated");
        setUser(user);
      } catch (err) {
        setStatus("unauthenticated");
        setUser(null);
        throw err;
      }
    },
    [status, setStatus, setUser, router]
  );

  const logout = useCallback(async () => {
    if (status === "unauthenticated" || status === "loading") return;

    setStatus("loading");

    try {
      const res = await fetch("/api/auth/logout", { method: "GET" });
      if (!res.ok) throw new Error("Logout failed");

      // Refresh the page
      startTransition(() => router.refresh());

      setStatus("unauthenticated");
      setUser(null);
    } catch (err) {
      setStatus("authenticated");
      throw err;
    }
  }, [status, setStatus, setUser, router]);

  // Get the initial authentication status
  useEffect(() => {
    getAuthStatus()
      .then((res) => {
        if (res.status === "authenticated") {
          setStatus("authenticated");
          setUser(res.user);
        } else {
          setStatus("unauthenticated");
        }
      })
      .catch(() => {
        setStatus("unauthenticated");
        setUser(null);
      });
  }, [setStatus, setUser]);

  return (
    <AuthContext.Provider value={{ loading, login, logout, status, user }}>
      {children}
    </AuthContext.Provider>
  );
}

export function useAuth() {
  return useContext(AuthContext);
}
