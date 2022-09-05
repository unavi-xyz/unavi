import { createContext, useEffect, useState } from "react";
import { useAccount } from "wagmi";

export interface ILensContext {
  handle: string | undefined;
  switchProfile: (handle: string | undefined) => void;
  logout: () => void;
}

export const initialContext: ILensContext = {
  handle: undefined,
  switchProfile: () => {},
  logout: () => {},
};

export const LensContext = createContext<ILensContext>(initialContext);

export default function LensProvider({
  children,
}: {
  children: React.ReactNode;
}) {
  const [handle, setHandle] = useState<string>();

  const { address, isConnected } = useAccount();

  useEffect(() => {
    if (isConnected && handle) {
      // Mark current handle as auto-login
      // sessionStorage.setItem(SessionStorage.AutoLogin, handle);
    }
  }, [isConnected, handle, setHandle]);

  function logout() {
    // sessionStorage.removeItem(SessionStorage.AutoLogin);
    setHandle(undefined);
  }

  return (
    <LensContext.Provider
      value={{
        handle,
        switchProfile: setHandle,
        logout,
      }}
    >
      {children}
    </LensContext.Provider>
  );
}
