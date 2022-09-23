import { createContext, useEffect, useMemo, useState } from "react";
import { Client, createClient, Provider } from "urql";
import { useAccount } from "wagmi";

import { SessionStorage } from "../../constants";
import { API_URL } from "./constants";

export interface ILensContext {
  client: Client;
  handle: string | undefined;
  switchProfile: (handle: string | undefined) => void;
  logout: () => void;
  setAccessToken: (accessToken: string) => void;
}

const defaultClient = createClient({ url: API_URL });

export const initialContext: ILensContext = {
  client: defaultClient,
  handle: undefined,
  switchProfile: () => {},
  logout: () => {},
  setAccessToken: () => {},
};

export const LensContext = createContext<ILensContext>(initialContext);

export default function LensProvider({
  children,
}: {
  children: React.ReactNode;
}) {
  const { isConnected } = useAccount();

  const [handle, setHandle] = useState<string>();
  const [client, setClient] = useState<Client>(defaultClient);

  useEffect(() => {
    if (isConnected && handle) {
      // Mark current handle as auto-login
      sessionStorage.setItem(SessionStorage.AutoLogin, handle);
    }
  }, [isConnected, handle, setHandle]);

  function logout() {
    sessionStorage.removeItem(SessionStorage.AutoLogin);
    setHandle(undefined);
  }

  const setAccessToken = useMemo(() => {
    return (accessToken: string) => {
      const newClient = createClient({
        url: API_URL,
        fetchOptions: () => {
          return {
            headers: { authorization: `Bearer ${accessToken}` },
          };
        },
      });

      setClient(newClient);
    };
  }, [setClient]);

  return (
    <Provider value={client}>
      <LensContext.Provider
        value={{
          client,
          handle,
          switchProfile: setHandle,
          logout,
          setAccessToken,
        }}
      >
        {children}
      </LensContext.Provider>
    </Provider>
  );
}
