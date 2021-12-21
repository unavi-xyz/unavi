import React, { useState } from "react";
import sdk from "matrix-js-sdk";

const withHttps = (url: string) =>
  !/^https?:\/\//i.test(url) ? `https://${url}` : url;

export const MatrixContext = React.createContext({
  loggedIn: false,
  login: async (homeserver: string, user: string, password: string) =>
    undefined,
  register: async (homeserver: string, user: string, password: string) =>
    undefined,
  logout: () => {},
});

export default function MatrixProvider({ children }) {
  const [loggedIn, setLoggedIn] = useState(false);

  async function login(homeserver: string, user: string, password: string) {
    try {
      const client = sdk.createClient(withHttps(homeserver));
      const { user_id, device_id, access_token } = await client.login(
        "m.login.password",
        {
          user,
          password,
        }
      );

      localStorage.setItem(
        "matrix-auth-store",
        JSON.stringify({
          user_id,
          device_id,
          access_token,
        })
      );

      setLoggedIn(true);

      return;
    } catch (e) {
      return e;
    }
  }

  async function register(
    homeserver: string,
    username: string,
    password: string
  ) {}

  function logout() {
    setLoggedIn(false);
  }

  return (
    <MatrixContext.Provider value={{ loggedIn, login, logout, register }}>
      {children}
    </MatrixContext.Provider>
  );
}
