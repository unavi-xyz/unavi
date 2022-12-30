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
      const [author, metadata] = await Promise.all([
        getSpaceAuthor(input.id),
        getSpaceMetadata(input.id),
      ]);

      return {
        id: input.id,
        author,
        metadata,
      };
    }),

  latest: publicProcedure
    .input(
      z.object({
        limit: z.number().min(1).max(99).optional().default(15),
        owner: z.string().optional(),
      })
    )
    .query(async ({ input }) => {
      const spaceContract = Space__factory.connect(SPACE_ADDRESS, ethersProvider);

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

      const count = (await spaceContract.count()).toNumber();

      // Get latest spaces
      const initialIds = new Array(Math.min(input.limit, count)).fill(0).map((_, i) => count - i);
      const spaces: ValidResponse[] = [];

      await Promise.all(
        initialIds.map(async (id) => {
          const valid = await isSpaceValid(id);
          if (valid) spaces.push(valid);
        })
      );

      // If any spaces were invalid, loop backwards until we have enough
      let i = input.limit;

      while (spaces.length < input.limit) {
        const tokenId = count - i;
        i++;

        // Break if we've reached the end
        if (tokenId <= 0) break;

        const valid = await isSpaceValid(tokenId);
        if (valid) spaces.push(valid);
      }

      return spaces;
    }),
});
