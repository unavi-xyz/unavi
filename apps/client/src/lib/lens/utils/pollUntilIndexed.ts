import {
  HasTxHashBeenIndexedDocument,
  HasTxHashBeenIndexedQuery,
  HasTxHashBeenIndexedQueryVariables,
} from "@wired-labs/lens";

import { lensClient } from "../client";

export async function pollUntilIndexed(txHash: string) {
  while (true) {
    const { data, error } = await lensClient
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
