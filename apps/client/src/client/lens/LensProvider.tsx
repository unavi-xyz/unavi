import { useEffect, useState } from "react";
import { Client, createClient, Provider } from "urql";

import { API_URL } from "./constants";
import { initialContext, LensContext } from "./context";

export default function LensProvider({ children }: { children: React.ReactNode }) {
  const [handle, setHandle] = useState<string>();
  const [client, setClient] = useState<Client>(initialContext.client);
  const [accessToken, setAccessToken] = useState<string>();

  useEffect(() => {
    if (!accessToken) {
      setClient(initialContext.client);
      return;
    }

    const newClient = createClient({
      url: API_URL,
      fetchOptions: () => {
        return {
          headers: { "x-access-token": accessToken },
        };
      },
    });

    setClient(newClient);
  }, [accessToken]);

  return (
    <Provider value={client}>
      <LensContext.Provider
        value={{
          client,
          handle,
          switchProfile: setHandle,
          setAccessToken,
        }}
      >
        {children}
      </LensContext.Provider>
    </Provider>
  );
}
