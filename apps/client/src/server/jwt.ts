import {
  AuthenticateDocument,
  AuthenticateMutation,
  AuthenticateMutationVariables,
  VerifyDocument,
  VerifyQuery,
  VerifyQueryVariables,
} from "@wired-labs/lens";

import { lensClient } from "./lens";

export async function authenticate(address: string, signature: string) {
  const { data, error } = await lensClient
    .mutation<AuthenticateMutation, AuthenticateMutationVariables>(
      AuthenticateDocument,
      {
        request: {
          address,
          signature,
        },
      }
    )
    .toPromise();

  if (error) throw new Error(error.message);
  if (!data) throw new Error("Authentication failed");

  return {
    accessToken: data.authenticate.accessToken as string,
    refreshToken: data.authenticate.refreshToken as string,
  };
}

export async function verifyJWT(accessToken: string) {
  const { data, error } = await lensClient
    .query<VerifyQuery, VerifyQueryVariables>(VerifyDocument, {
      request: { accessToken },
    })
    .toPromise();

  if (error) throw new Error(error.message);
  if (data === undefined) throw new Error("No data recieved");

  return data.verify;
}
