import {
  GetChallengeDocument,
  GetChallengeQuery,
  GetChallengeQueryVariables,
} from "@wired-labs/lens";
import { useEffect, useMemo, useState } from "react";
import { Client } from "urql";

import { useLens } from "../lib/lens/hooks/useLens";

export function useChallenge(address: string | undefined) {
  const { client } = useLens();

  const challengePromise = useMemo(
    async () => await generateChallenge(address, client),
    [address, client]
  );

  const [challenge, setChallenge] = useState<string>();

  useEffect(() => {
    challengePromise.then(setChallenge);
  }, [challengePromise]);

  return challenge;
}

async function generateChallenge(address: string | undefined, client: Client) {
  if (!address) return;

  const { data, error } = await client
    .query<GetChallengeQuery, GetChallengeQueryVariables>(
      GetChallengeDocument,
      {
        request: { address },
      }
    )
    .toPromise();

  if (error) throw new Error(error.message);
  if (data === undefined) throw new Error("No challenge recieved");

  return data.challenge.text;
}
