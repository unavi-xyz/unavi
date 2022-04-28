import { apolloClient } from "./apollo";
import { AUTHENTICATE, GET_CHALLENGE, REFRESH_AUTHENTICATION } from "./queries";
import { LOCAL_STORAGE } from "./constants";
import { useLensStore } from "./store";
import { useEthersStore } from "../ethers/store";
import { disconnectWallet } from "../ethers/connection";

import {
  AuthenticateMutation,
  AuthenticateMutationVariables,
  GetChallengeQuery,
  GetChallengeQueryVariables,
  RefreshAuthenticationMutation,
  RefreshAuthenticationMutationVariables,
} from "../../generated/graphql";

async function setAccessToken(accessToken: string) {
  const address = useEthersStore.getState().address;
  const ThirtyMinutesFromNow = new Date(new Date().getTime() + 30 * 60 * 1000);

  localStorage.setItem(`${address}${LOCAL_STORAGE.ACCESS_TOKEN}`, accessToken);
  localStorage.setItem(
    `${address}${LOCAL_STORAGE.ACCESS_EXPIRE}`,
    ThirtyMinutesFromNow.toString()
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
    const { data } = await apolloClient.mutate<
      RefreshAuthenticationMutation,
      RefreshAuthenticationMutationVariables
    >({
      mutation: REFRESH_AUTHENTICATION,
      variables: { refreshToken },
    });

    //store tokens
    if (data) {
      setAccessToken(data.refresh.accessToken);
      setRefreshToken(data.refresh.refreshToken);
    }
  }
}

async function generateChallenge(address: string) {
  const { data } = await apolloClient.query<
    GetChallengeQuery,
    GetChallengeQueryVariables
  >({
    query: GET_CHALLENGE,
    variables: { address },
  });

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

  //if no valid token, generate challenge
  const challenge = await generateChallenge(address);

  //sign challenge
  const signature = await signer.signMessage(challenge);

  //authenticate with api
  const { data } = await apolloClient.mutate<
    AuthenticateMutation,
    AuthenticateMutationVariables
  >({
    mutation: AUTHENTICATE,
    variables: { address, signature },
  });

  if (!data) throw new Error("No data returned from authentication");

  //store tokens
  setAccessToken(data.authenticate.accessToken);
  setRefreshToken(data.authenticate.refreshToken);

  useLensStore.setState({ authenticated: true });
}

export function logout() {
  useLensStore.setState({ authenticated: false, handle: undefined });
  disconnectWallet();
}
