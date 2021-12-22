import React, { useEffect, useState } from "react";
import sdk, { MatrixClient } from "matrix-js-sdk";

const withHttps = (url: string) =>
  !/^https?:\/\//i.test(url) ? `https://${url}` : url;

export const MatrixContext = React.createContext({
  loggedIn: false,
  userId: "",
  client: null,
  login: async (homeserver: string, user: string, password: string) =>
    undefined,
  register: async (homeserver: string, user: string, password: string) =>
    undefined,
  logout: () => {},
});

export default function MatrixProvider({ children }) {
  const [loggedIn, setLoggedIn] = useState(false);
  const [userId, setUserId] = useState("");
  const [client, setClient] = useState<null | MatrixClient>(null);

  async function login(homeserver: string, username: string, password: string) {
    try {
      const tmpClient = sdk.createClient(withHttps(homeserver));
      const { user_id, device_id, access_token } =
        await tmpClient.loginWithPassword(username, password);

      localStorage.setItem(
        "matrix-auth-store",
        JSON.stringify({
          homeserver: withHttps(homeserver),
          user_id,
          device_id,
          access_token,
        })
      );

      setClient(tmpClient);
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
    try {
      const tmpClient = sdk.createClient(withHttps(homeserver));
      const { user_id, device_id, access_token } = await tmpClient.register(
        username,
        password,
        null,
        {
          type: "m.login.dummy",
        }
      );

      localStorage.setItem(
        "matrix-auth-store",
        JSON.stringify({
          homeserver: withHttps(homeserver),
          user_id,
          device_id,
          access_token,
        })
      );

      setClient(tmpClient);
      setUserId(user_id);
      setLoggedIn(true);

      return;
    } catch (e) {
      logout();
      return e;
    }
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
      store.homeserver &&
      store.user_id &&
      store.device_id &&
      store.access_token
    ) {
      setClient(sdk.createClient(String(store.homeserver)));
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
