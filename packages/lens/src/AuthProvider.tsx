import { createContext, useContext, useEffect, useState } from "react";
import { useAccount, useSignMessage } from "wagmi";

import {
  AuthenticateDocument,
  AuthenticateMutation,
  AuthenticateMutationVariables,
  GetChallengeDocument,
  GetChallengeQuery,
  GetChallengeQueryVariables,
  RefreshDocument,
  RefreshMutation,
  RefreshMutationVariables,
} from "../generated/graphql";
import { ClientContext } from "./ClientProvider";
import { LocalStorage, SessionStorage } from "./constants";

export interface IAuthContext {
  authenticated: boolean;
  handle: string | undefined;
  authenticate: () => Promise<void>;
  logout: () => void;
  setHandle: (handle: string | undefined) => void;
}

export const initialAuthContext: IAuthContext = {
  authenticated: false,
  handle: undefined,
  authenticate: () => Promise.resolve(),
  logout: () => {},
  setHandle: () => {},
};

export const AuthContext = createContext<IAuthContext>(initialAuthContext);

export function AuthProvider({ children }: { children: React.ReactNode }) {
  const [authenticated, setAuthenticated] = useState(false);
  const [handle, setHandle] = useState<string>();

  const { client } = useContext(ClientContext);
  const { address, isConnected } = useAccount();
  const { signMessageAsync } = useSignMessage();

  useEffect(() => {
    if (isConnected && handle) {
      // Mark current handle as auto-login
      sessionStorage.setItem(SessionStorage.AutoLogin, handle);
    }
  }, [isConnected, handle, setHandle]);

  async function generateChallenge(address: string) {
    const { data, error } = await client
      .query<GetChallengeQuery, GetChallengeQueryVariables>(GetChallengeDocument, {
        request: {
          address,
        },
      })
      .toPromise();

    if (error) throw new Error(error.message);
    if (!data) throw new Error("No challenge recieved");

    return data.challenge.text;
  }

  async function setAccessToken(accessToken: string) {
    const TwentyNineMinutesFromNow = new Date(new Date().getTime() + 29 * 60 * 1000);

    localStorage.setItem(`${LocalStorage.AccessToken}${address}`, accessToken);
    localStorage.setItem(
      `${LocalStorage.AccessExpire}${address}`,
      TwentyNineMinutesFromNow.toString()
    );
  }

  async function setRefreshToken(refreshToken: string) {
    const OneDayFromNow = new Date(new Date().getTime() + 24 * 60 * 60 * 1000);

    localStorage.setItem(`${LocalStorage.RefreshToken}${address}`, refreshToken);
    localStorage.setItem(`${LocalStorage.RefreshExpire}${address}`, OneDayFromNow.toString());
  }

  async function refreshAccessToken() {
    const refreshToken = localStorage.getItem(`${LocalStorage.RefreshToken}${address}`);

    if (refreshToken) {
      const { data, error } = await client
        .mutation<RefreshMutation, RefreshMutationVariables>(RefreshDocument, {
          request: {
            refreshToken,
          },
        })
        .toPromise();

      if (error) throw new Error(error.message);
      if (!data) throw new Error("No refresh tokens recieved");

      //store tokens
      setAccessToken(data.refresh.accessToken);
      setRefreshToken(data.refresh.refreshToken);
    }
  }

  async function authenticate() {
    if (!address) throw new Error("No address");

    //if access token is valid, return
    const accessToken = localStorage.getItem(`${LocalStorage.AccessToken}${address}`);
    const accessExpire = localStorage.getItem(`${LocalStorage.AccessExpire}${address}`);

    if (accessToken && accessExpire) {
      const now = new Date();
      const expire = new Date(accessExpire);
      if (now < expire) {
        return;
      }
    }

    //if refresh token is valid, refresh access token
    const refreshToken = localStorage.getItem(`${LocalStorage.RefreshToken}${address}`);
    const refreshExpire = localStorage.getItem(`${LocalStorage.RefreshExpire}${address}`);

    if (refreshToken && refreshExpire) {
      const now = new Date();
      const expire = new Date(refreshExpire);
      if (now < expire) {
        await refreshAccessToken();
        return;
      }
    }

    //if no valid token, generate a challenge
    const message = await generateChallenge(address);

    //sign message
    const signature = await signMessageAsync({ message });

    //authenticate with api
    const { data, error } = await client
      .mutation<AuthenticateMutation, AuthenticateMutationVariables>(AuthenticateDocument, {
        request: {
          address,
          signature,
        },
      })
      .toPromise();

    if (error) throw new Error(error.message);
    if (!data) throw new Error("Authentication failed");

    //store tokens
    setAccessToken(data.authenticate.accessToken);
    setRefreshToken(data.authenticate.refreshToken);

    setAuthenticated(true);
  }

  function logout() {
    sessionStorage.removeItem(SessionStorage.AutoLogin);
    setAuthenticated(false);
    setHandle(undefined);
  }

  return (
    <AuthContext.Provider
      value={{
        authenticated,
        handle,
        authenticate,
        logout,
        setHandle,
      }}
    >
      {children}
    </AuthContext.Provider>
  );
}
