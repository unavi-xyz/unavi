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
} from "../../generated/graphql";
import { disconnectWallet } from "../ethers/connection";
import { useEthersStore } from "../ethers/store";
import { AUTO_LOGIN_KEY } from "../ethers/useAutoLogin";
import { lensClient } from "./client";
import { LOCAL_STORAGE } from "./constants";
import { useLensStore } from "./store";

async function setAccessToken(accessToken: string) {
  const address = useEthersStore.getState().address;
  const TwentyNineMinutesFromNow = new Date(
    new Date().getTime() + 29 * 60 * 1000
  );

  localStorage.setItem(`${address}${LOCAL_STORAGE.ACCESS_TOKEN}`, accessToken);
  localStorage.setItem(
    `${address}${LOCAL_STORAGE.ACCESS_EXPIRE}`,
    TwentyNineMinutesFromNow.toString()
  );
}

async function setRefreshToken(refreshToken: string) {
  const address = useEthersStore.getState().address;
  const OneDayFromNow = new Date(new Date().getTime() + 24 * 60 * 60 * 1000);

  localStorage.setItem(
    `${address}${LOCAL_STORAGE.REFRESH_TOKEN}`,
    refreshToken
  );
  localStorage.setItem(
    `${address}${LOCAL_STORAGE.REFRESH_EXPIRE}`,
    OneDayFromNow.toString()
  );
}

export async function refreshAccessToken() {
  const address = useEthersStore.getState().address;
  const refreshToken = localStorage.getItem(
    `${address}${LOCAL_STORAGE.REFRESH_TOKEN}`
  );

  if (refreshToken) {
    const { data, error } = await lensClient
      .mutation<RefreshMutation, RefreshMutationVariables>(RefreshDocument, {
        refreshToken,
      })
      .toPromise();

    if (error) throw new Error(error.message);
    if (!data) throw new Error("No refresh tokens recieved");

    //store tokens
    setAccessToken(data.refresh.accessToken);
    setRefreshToken(data.refresh.refreshToken);
  }
}

async function generateChallenge(address: string) {
  const { data, error } = await lensClient
    .query<GetChallengeQuery, GetChallengeQueryVariables>(
      GetChallengeDocument,
      { address }
    )
    .toPromise();

  if (error) throw new Error(error.message);
  if (!data) throw new Error("No challenge recieved");

  return data.challenge.text;
}

export async function authenticate() {
  const address = useEthersStore.getState().address;
  const signer = useEthersStore.getState().signer;

  if (!address) throw new Error("No address");
  if (!signer) throw new Error("No signer");

  //if access token is valid, return
  const accessToken = localStorage.getItem(
    `${address}${LOCAL_STORAGE.ACCESS_TOKEN}`
  );
  const accessExpire = localStorage.getItem(
    `${address}${LOCAL_STORAGE.ACCESS_EXPIRE}`
  );

  if (accessToken && accessExpire) {
    const now = new Date();
    const expire = new Date(accessExpire);
    if (now < expire) {
      return;
    }
  }

  //if refresh token is valid, refresh access token
  const refreshToken = localStorage.getItem(
    `${address}${LOCAL_STORAGE.REFRESH_TOKEN}`
  );
  const refreshExpire = localStorage.getItem(
    `${address}${LOCAL_STORAGE.REFRESH_EXPIRE}`
  );

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
  const { data, error } = await lensClient
    .mutation<AuthenticateMutation, AuthenticateMutationVariables>(
      AuthenticateDocument,
      { address, signature }
    )
    .toPromise();

  if (error) throw new Error(error.message);
  if (!data) throw new Error("Authentication failed");

  //store tokens
  setAccessToken(data.authenticate.accessToken);
  setRefreshToken(data.authenticate.refreshToken);

  useLensStore.setState({ authenticated: true });
}

export function logout() {
  sessionStorage.removeItem(AUTO_LOGIN_KEY);
  useLensStore.setState({ authenticated: false, handle: undefined });
  disconnectWallet();
}
