import { AuthenticationStatus } from "@rainbow-me/rainbowkit";
import { User } from "lucia-auth";
import { create } from "zustand";

export interface AuthStore {
  status: AuthenticationStatus;
  setStatus: (status: AuthenticationStatus) => void;
  user: User | null;
  setUser: (user: User | null) => void;
  setUsername: (username: string) => void;
}

export const useAuthStore = create<AuthStore>((set, get) => ({
  setStatus: (status: AuthenticationStatus) => set({ status }),
  setUser: (user: User | null) => set({ user }),
  setUsername: (username: string) => {
    const user = get().user;
    if (!user) return;
    user.username = username;
    set({ user });
  },
  status: "loading",
  user: null,
}));
