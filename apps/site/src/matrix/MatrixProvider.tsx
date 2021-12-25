import React, { useEffect, useState } from "react";
import sdk, { MatrixClient, ICreateClientOpts } from "matrix-js-sdk";

const withHttps = (url: string) =>
  !/^https?:\/\//i.test(url) ? `https://${url}` : url;

function initClient(
  baseUrl: string,
  accessToken: string,
  deviceId: string,
  userId: string
) {
  const opts: ICreateClientOpts = { baseUrl, accessToken, deviceId, userId };
  const client = sdk.createClient(opts);
  return client;
}

interface ContextInterface {
  loggedIn: boolean;
  userId: null | string;
  client: null | MatrixClient;
  login: (
    homeserver: string,
    user: string,
    password: string
  ) => Promise<undefined | Error>;
  register: (
    homeserver: string,
    user: string,
    password: string
  ) => Promise<undefined | Error>;
  logout: () => void;
}

const defaultContext: ContextInterface = {
  loggedIn: false,
  userId: null,
  client: null,
  login: null,
  register: null,
  logout: null,
};

export const MatrixContext = React.createContext(defaultContext);

export default function MatrixProvider({ children }) {
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

      setClient(initClient(baseUrl, access_token, device_id, user_id));
      setUserId(user_id);
      setLoggedIn(true);

      return;
    } catch (e) {
      logout();
      return e;
    }
  }

  async function register(
    homeserver: string,
    username: string,
    password: string
  ) {
    return Promise.reject();
  }

  function logout() {
    localStorage.removeItem("matrix-auth-store");
    setClient(null);
    setUserId("");
    setLoggedIn(false);
  }

  useEffect(() => {
    const store = JSON.parse(localStorage.getItem("matrix-auth-store"));
    if (
      store &&
      store.baseUrl &&
      store.access_token &&
      store.device_id &&
      store.user_id
    ) {
      setClient(
        initClient(
          store.baseUrl,
          store.access_token,
          store.device_id,
          store.user_id
        )
      );
      setUserId(String(store.user_id));
      setLoggedIn(true);
    } else {
      logout();
    }
  }, []);

  return (
    <MatrixContext.Provider
      value={{ loggedIn, userId, client, login, logout, register }}
    >
      {children}
    </MatrixContext.Provider>
  );
}
