import { createContext, useContext, useState } from "react";

import { EthersContext } from "@wired-xr/ethers";

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
  setHandle: (handle: string | undefined) => void;
  authenticate: () => Promise<void>;
  logout: () => void;
  switchProfile: (handle: string) => void;
}

export const initialAuthContext: IAuthContext = {
  authenticated: false,
  handle: undefined,
  setHandle: () => {},
  authenticate: () => Promise.resolve(),
  logout: () => {},
  switchProfile: () => {},
};

export const AuthContext = createContext<IAuthContext>(initialAuthContext);

export function AuthProvider({ children }: { children: React.ReactNode }) {
  const { client } = useContext(ClientContext);
  const { address, signer } = useContext(EthersContext);

  const [authenticated, setAuthenticated] = useState(false);
  const [handle, setHandle] = useState<string>();

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
    if (!signer) throw new Error("No signer");

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
    const challenge = await generateChallenge(address);

    //sign challenge
    const signature = await signer.signMessage(challenge);

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

  function switchProfile(handle: string) {
    sessionStorage.setItem(SessionStorage.AutoLogin, handle);
    localStorage.setItem(`${LocalStorage.PreviousHandle}${address}`, handle);
    setHandle(handle);
  }

  return (
    <AuthContext.Provider
      value={{
        authenticated,
        handle,
        setHandle,
        authenticate,
        logout,
        switchProfile,
      }}
    >
      {children}
    </AuthContext.Provider>
  );
}
