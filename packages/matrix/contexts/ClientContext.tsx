import React, { ReactChild, useEffect, useState } from "react";
import sdk, { MatrixClient, ICreateClientOpts } from "matrix-js-sdk";
import { DEFAULT_HOMESERVER } from "..";

function withHttps(url: string) {
  return !/^https?:\/\//i.test(url) ? `https://${url}` : url;
}

function waitForSync(client: MatrixClient) {
  return new Promise<void>((resolve) => {
    function onSync(state: string) {
      if (state === "PREPARED") {
        client.removeListener("sync", onSync);
        resolve();
      }
    }
    client.on("sync", onSync);
  });
}

export async function initClient(
  baseUrl: string,
  accessToken: string,
  deviceId: string,
  userId: string,
  guest: boolean = false
) {
  const opts: ICreateClientOpts = { baseUrl, accessToken, deviceId, userId };
  const client = sdk.createClient(opts);

  if (guest) client.setGuest(true);

  await client.startClient({});

  await waitForSync(client);

  return client;
}

interface ContextInterface {
  loggedIn: boolean;
  userId: string;
  client: MatrixClient;
  login:
    | null
    | ((
        homeserver: string,
        user: string,
        password: string
      ) => Promise<undefined | Error>);
  logout: () => void;
}

const defaultContext: ContextInterface = {
  loggedIn: false,
  userId: "",
  client: sdk.createClient(""),
  login: null,
  logout: () => {},
};

export const ClientContext = React.createContext(defaultContext);

export function ClientProvider({ children }: { children: ReactChild }) {
  const [loggedIn, setLoggedIn] = useState(defaultContext.loggedIn);
  const [userId, setUserId] = useState(defaultContext.userId);
  const [client, setClient] = useState(defaultContext.client);

  async function login(homeserver: string, username: string, password: string) {
    try {
      const baseUrl = withHttps(homeserver);
      const tmpClient = sdk.createClient(baseUrl);
      const { user_id, device_id, access_token } =
        await tmpClient.loginWithPassword(username, password);

      localStorage.setItem(
        "matrix-auth-store",
        JSON.stringify({
          baseUrl,
          user_id,
          device_id,
          access_token,
        })
      );

      const newClient = await initClient(
        baseUrl,
        access_token,
        device_id,
        user_id
      );

      setClient(newClient);
      setUserId(user_id);
      setLoggedIn(true);

      return;
    } catch (e) {
      logout();
      return e as Error;
    }
  }

  async function registerGuest() {
    const tmpClient = sdk.createClient(DEFAULT_HOMESERVER);
    const { user_id, device_id, access_token } = await tmpClient.registerGuest(
      {}
    );

    const newClient = await initClient(
      DEFAULT_HOMESERVER,
      access_token,
      device_id,
      user_id,
      true
    );

    setClient(newClient);
  }

  function logout() {
    localStorage.removeItem("matrix-auth-store");
    setClient(defaultContext.client);
    setUserId(defaultContext.userId);
    setLoggedIn(false);
    registerGuest();
  }

  useEffect(() => {
    const store = JSON.parse(localStorage.getItem("matrix-auth-store") ?? "{}");
    if (
      store &&
      store.baseUrl &&
      store.access_token &&
      store.device_id &&
      store.user_id
    ) {
      initClient(
        store.baseUrl,
        store.access_token,
        store.device_id,
        store.user_id
      ).then((newClient) => {
        setClient(newClient);
        setUserId(String(store.user_id));
        setLoggedIn(true);
      });
    } else {
      logout();
    }
  }, []);

  return (
    <ClientContext.Provider value={{ loggedIn, userId, client, login, logout }}>
      {children}
    </ClientContext.Provider>
  );
}
