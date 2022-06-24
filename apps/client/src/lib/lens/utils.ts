import {
  HasTxHashBeenIndexedDocument,
  HasTxHashBeenIndexedQuery,
  HasTxHashBeenIndexedQueryVariables,
} from "../../generated/graphql";
import { lensClient } from "./client";
import { HANDLE_ENDING } from "./constants";

export function removeTypename(obj: any) {
  if (obj.__typename) delete obj.__typename;

  Object.values(obj).forEach((value) => {
    if (typeof value === "object") removeTypename(value);
  });

  return obj;
}

export async function pollUntilIndexed(txHash: string) {
  while (true) {
    const { data, error } = await lensClient
      .query<HasTxHashBeenIndexedQuery, HasTxHashBeenIndexedQueryVariables>(
        HasTxHashBeenIndexedDocument,
        { txHash }
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
