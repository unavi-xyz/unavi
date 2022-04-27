import { apolloClient } from "./apollo";
import { AUTHENTICATE, GET_CHALLENGE, REFRESH_AUTHENTICATION } from "./queries";
import { useLensStore } from "./store";
import { useEthersStore } from "../ethers/store";
import { LOCAL_STORAGE } from "./constants";

import {
  AuthenticateMutation,
  AuthenticateMutationVariables,
  GetChallengeQuery,
  GetChallengeQueryVariables,
  RefreshAuthenticationMutation,
  RefreshAuthenticationMutationVariables,
} from "../../generated/graphql";

async function setAccessToken(accessToken: string) {
  localStorage.setItem(LOCAL_STORAGE.ACCESS_TOKEN, accessToken);
  const ThirtyMinutesFromNow = new Date(new Date().getTime() + 30 * 60 * 1000);
  localStorage.setItem(
    LOCAL_STORAGE.ACCESS_EXPIRE,
    ThirtyMinutesFromNow.toString()
  );
}

async function setRefreshToken(refreshToken: string) {
  localStorage.setItem(LOCAL_STORAGE.REFRESH_TOKEN, refreshToken);
  const OneDayFromNow = new Date(new Date().getTime() + 24 * 60 * 60 * 1000);
  localStorage.setItem(LOCAL_STORAGE.REFRESH_EXPIRE, OneDayFromNow.toString());
}

export async function refreshAccessToken() {
  const refreshToken = localStorage.getItem(LOCAL_STORAGE.REFRESH_TOKEN);
  if (refreshToken) {
    const { data } = await apolloClient.mutate<
      RefreshAuthenticationMutation,
      RefreshAuthenticationMutationVariables
    >({
      mutation: REFRESH_AUTHENTICATION,
      variables: { refreshToken },
    });

    //store tokens
    setAccessToken(data.refresh.accessToken);
    setRefreshToken(data.refresh.refreshToken);
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
  //if access token is valid, return
  const accessToken = localStorage.getItem(LOCAL_STORAGE.ACCESS_TOKEN);
  const accessExpire = localStorage.getItem(LOCAL_STORAGE.ACCESS_EXPIRE);
  if (accessToken && accessExpire) {
    const now = new Date();
    const expire = new Date(accessExpire);
    if (now < expire) {
      return;
    }
  }

  //if refresh token is valid, refresh access token
  const refreshToken = localStorage.getItem(LOCAL_STORAGE.REFRESH_TOKEN);
  const refreshExpire = localStorage.getItem(LOCAL_STORAGE.REFRESH_EXPIRE);
  if (refreshToken && refreshExpire) {
    const now = new Date();
    const expire = new Date(refreshExpire);
    if (now < expire) {
      await refreshAccessToken();
      return;
    }
  }

  //if no valid token, generate challenge
  const signer = useEthersStore.getState().signer;
  const address = useEthersStore.getState().address;

  const challenge = await generateChallenge(address);

  //sign challenge
  const signature = await signer.signMessage(challenge);

  //authenticate with server
  const { data } = await apolloClient.mutate<
    AuthenticateMutation,
    AuthenticateMutationVariables
  >({
    mutation: AUTHENTICATE,
    variables: { address, signature },
  });

  //store tokens
  setAccessToken(data.authenticate.accessToken);
  setRefreshToken(data.authenticate.refreshToken);

  useLensStore.setState({ authenticated: true });
}
