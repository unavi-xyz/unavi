import {
  HasTxHashBeenIndexedDocument,
  HasTxHashBeenIndexedQuery,
  HasTxHashBeenIndexedQueryVariables,
} from "@wired-labs/lens";

import { lensClient } from "../../../server/lens";

export async function pollUntilIndexed(txHash: string) {
  return new Promise<void>((resolve, reject) => {
    const interval = setInterval(async () => {
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

      if (error) reject(error);

      if (data?.hasTxHashBeenIndexed) {
        clearInterval(interval);
        resolve();
      }
    }, 1000);
  });
}
