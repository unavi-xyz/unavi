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
        owner: z.string().optional(),
      })
    )
    .query(async ({ input }) => {
      const spaceContract = Space__factory.connect(SPACE_ADDRESS, ethersProvider);

      const totalSupplyBigNumber = await spaceContract.totalSupply();
      const totalSupply = totalSupplyBigNumber.toNumber();

      const spaces = [];
      let i = 0;

      while (spaces.length < 20) {
        const tokenId = totalSupply - i;
        i++;

        // Break if we've reached the end
        if (tokenId <= 0) break;

        // Check if owner matches
        if (input.owner) {
          const owner = await spaceContract.ownerOf(tokenId);
          if (owner !== input.owner) continue;
        }

        // Check if metadata exists
        const metadata = await getSpaceMetadata(tokenId);
        if (!metadata) continue;

        spaces.push({ id: tokenId, metadata });
      }

      return spaces;
    }),
});
