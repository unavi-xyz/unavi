import { Space__factory, SPACE_ADDRESS } from "contracts";
import { z } from "zod";

import { ethersProvider } from "../constants";
import { getSpaceAuthor } from "../helpers/getSpaceAuthor";
import { getSpaceMetadata } from "../helpers/getSpaceMetadata";
import { publicProcedure, router } from "./trpc";

export const spaceRouter = router({
  byId: publicProcedure
    .input(
      z.object({
        id: z.number(),
      })
    )
    .query(async ({ input }) => {
      try {
        const [{ address, profile }, metadata] = await Promise.all([
          getSpaceAuthor(input.id),
          getSpaceMetadata(input.id),
        ]);

        return {
          id: input.id,
          owner: address,
          author: profile,
          metadata,
        };
      } catch {
        return null;
      }
    }),

  latest: publicProcedure
    .input(
      z.object({
        limit: z.number().min(1).max(99),
        owner: z.string().optional(),
      })
    )
    .query(async ({ input }) => {
      const spaceContract = Space__factory.connect(SPACE_ADDRESS, ethersProvider);

      const count = (await spaceContract.count()).toNumber();

      const isSpaceValid = async (tokenId: number) => {
        try {
          // Check if owner matches
          if (input.owner) {
            const owner = await spaceContract.ownerOf(tokenId);
            if (owner !== input.owner) return;
          }

          // Check if metadata exists
          const metadata = await getSpaceMetadata(tokenId);
          if (!metadata) return;

          return { id: tokenId, metadata };
        } catch {
          // Ignore
        }
      };

      type ValidResponse = Exclude<Awaited<ReturnType<typeof isSpaceValid>>, undefined>;

      const spaces: ValidResponse[] = [];

      let nextSpaceId = count - 1;

      const fetchSpace = async () => {
        if (nextSpaceId === 0 || spaces.length === input.limit) return;

        const valid = await isSpaceValid(nextSpaceId--);

        if (valid) spaces.push(valid);
        else await fetchSpace();
      };

      const amountToFetch = Math.min(input.limit, count);

      await Promise.all(Array.from({ length: amountToFetch }).map(fetchSpace));

      return spaces;
    }),
});
