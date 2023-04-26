"use client";

import { AuthenticationStatus } from "@rainbow-me/rainbowkit";
import { User } from "lucia-auth";
import { useRouter } from "next/navigation";
import {
  Context,
  createContext,
  useCallback,
  useContext,
  useEffect,
  useState,
  useTransition,
} from "react";

import { LoginResponse } from "@/app/api/auth/login/types";
import { getAuthStatus } from "@/app/api/auth/status/helper";
import { AuthData } from "@/src/server/auth/types";

export type AuthContextValue = {
  status: AuthenticationStatus;
  loading: boolean; // Sseparate loading state for transitions
  user: User | null;
  login: (args: AuthData) => Promise<void>;
  logout: () => Promise<void>;
};

export const AuthContext: Context<AuthContextValue> = createContext<AuthContextValue>({
  status: "unauthenticated",
  loading: false,
  user: null,
  login: async () => {},
  logout: async () => {},
});

interface Props {
  children: React.ReactNode;
}

/**
 * A context provider for authentication
 */
export default function AuthProvider({ children }: Props) {
  const [status, setStatus] = useState<AuthenticationStatus>("loading");
  const [user, setUser] = useState<User | null>(null);

  const [loading, startTransition] = useTransition();

  const router = useRouter();

  const login = useCallback(
    async (args: AuthData) => {
      if (status === "authenticated" || status === "loading") return;

      setStatus("loading");

      try {
        const res = await fetch("/api/auth/login", {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify(args),
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
    [status, setStatus, router]
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
  }, [status, setStatus, router]);

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
  }, []);

  return (
    <AuthContext.Provider value={{ status, loading, user, login, logout }}>
      {children}
    </AuthContext.Provider>
  );
}

export function useAuth() {
  return useContext(AuthContext);
}
