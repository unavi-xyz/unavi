import { createContext } from "react";

import { SessionStorage } from "@wired-xr/lens";

import { useAuthenticate } from "./useAuthenticate";
import { useAutoLogin } from "./useAutoLogin";

export const LoginContext = createContext({
  authenticated: false,
  logout: () => {},
});

interface Props {
  children: React.ReactNode;
}

export default function LoginProvider({ children }: Props) {
  const { authenticated } = useAuthenticate();
  useAutoLogin();

  function logout() {
    sessionStorage.removeItem(SessionStorage.ActiveDatabaseToken);
    sessionStorage.removeItem(SessionStorage.AutoLogin);
    window.location.reload();
  }

  return (
    <LoginContext.Provider value={{ authenticated, logout }}>{children}</LoginContext.Provider>
  );
}
