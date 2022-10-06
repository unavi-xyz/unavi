import {
  HasTxHashBeenIndexedDocument,
  HasTxHashBeenIndexedQuery,
  HasTxHashBeenIndexedQueryVariables,
} from "@wired-labs/lens";
import { Client } from "urql";

export async function pollUntilIndexed(txHash: string, client: Client) {
  return new Promise<void>((resolve, reject) => {
    const interval = setInterval(async () => {
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

      if (error) reject(error);

      if (data?.hasTxHashBeenIndexed) {
        clearInterval(interval);
        resolve();
      }
    }, 2000);
  });
}
