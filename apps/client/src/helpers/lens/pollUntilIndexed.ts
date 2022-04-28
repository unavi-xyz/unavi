import { apolloClient } from "./apollo";
import { HAS_TX_HASH_BEEN_INDEXED } from "./queries";
import {
  HasTxHashBeenIndexedQuery,
  HasTxHashBeenIndexedQueryVariables,
} from "../../generated/graphql";

export async function pollUntilIndexed(txHash: string) {
  //poll query until transaction is indexed
  while (true) {
    const result = await apolloClient.query<
      HasTxHashBeenIndexedQuery,
      HasTxHashBeenIndexedQueryVariables
    >({
      query: HAS_TX_HASH_BEEN_INDEXED,
      variables: { txHash },
    });

    const response = result.data.hasTxHashBeenIndexed;

    if (response.__typename === "TransactionIndexedResult") {
      if (response.metadataStatus) {
        if (response.metadataStatus.status === "SUCCESS") {
          return response;
        }

        if (response.metadataStatus.status === "METADATA_VALIDATION_FAILED") {
          throw new Error(
            response.metadataStatus.reason ?? response.metadataStatus.status
          );
        }
      } else {
        if (response.indexed) {
          return response;
        }
      }

      // sleep for a second before trying again
      await new Promise((resolve) => setTimeout(resolve, 500));
    }
  }
}
