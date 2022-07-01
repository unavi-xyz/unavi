import { Client } from "urql";

import {
  HasTxHashBeenIndexedDocument,
  HasTxHashBeenIndexedQuery,
  HasTxHashBeenIndexedQueryVariables,
} from "../generated/graphql";
import { HANDLE_ENDING } from "./constants";

export function removeTypename(obj: any) {
  if (obj.__typename) delete obj.__typename;

  Object.values(obj).forEach((value) => {
    if (typeof value === "object") removeTypename(value);
  });

  return obj;
}

export async function pollUntilIndexed(client: Client, txHash: string) {
  while (true) {
    const { data, error } = await client
      .query<HasTxHashBeenIndexedQuery, HasTxHashBeenIndexedQueryVariables>(
        HasTxHashBeenIndexedDocument,
        {
          request: {
            txHash,
          },
        }
      )
      .toPromise();

    if (error) throw new Error(error.message);
    if (data?.hasTxHashBeenIndexed) return;

    await new Promise((resolve) => setTimeout(resolve, 1000));
  }
}

export function trimHandle(handle: string) {
  if (!handle) return handle;
  return handle.substring(0, handle.length - HANDLE_ENDING.length);
}
