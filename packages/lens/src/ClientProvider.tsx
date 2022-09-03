import { createContext, useEffect, useState } from "react";
import { Client, Provider, createClient } from "urql";
import { useAccount } from "wagmi";

import { API_URL, LocalStorage } from "./constants";

const defaultClient = createClient({ url: API_URL });

export interface IClientContext {
  client: Client;
}

export const initialClientContext: IClientContext = { client: defaultClient };
export const ClientContext =
  createContext<IClientContext>(initialClientContext);

interface Props {
  children: React.ReactNode;
}

export function ClientProvider({ children }: Props) {
  const [client, setClient] = useState<Client>(defaultClient);

  const { address } = useAccount();

  useEffect(() => {
    if (!address) {
      setClient(defaultClient);
      return;
    }

    function getAccessToken() {
      if (!address) return;

      const accessToken = localStorage.getItem(
        `${LocalStorage.AccessToken}${address}`
      );

      return accessToken;
    }

    const client = createClient({
      url: API_URL,
      fetchOptions: () => {
        const token = getAccessToken();

        return {
          headers: { authorization: token ? `Bearer ${token}` : "" },
        };
      },
    });

    setClient(client);
  }, [address]);

  return (
    <Provider value={client}>
      <ClientContext.Provider
        value={{
          client,
        }}
      >
        {children}
      </ClientContext.Provider>
    </Provider>
  );
}
