import { ERC721MetadataSchema, Space__factory, SPACE_ADDRESS } from "contracts";
import { z } from "zod";

import { ethersProvider } from "../constants";
import { publicProcedure, router } from "./trpc";

export const spaceRouter = router({
  byId: publicProcedure
    .input(
      z.object({
        id: z.number(),
      })
    )
    .query(async ({ input }) => {
      const contract = Space__factory.connect(SPACE_ADDRESS, ethersProvider);

      const owner = await contract.ownerOf(input.id);
      const tokenURI = await contract.tokenURI(input.id);

      if (!tokenURI) return null;

      try {
        const response = await fetch(tokenURI);
        const json = await response.json();

        const metadata = ERC721MetadataSchema.parse(json);

        return {
          id: input.id,
          owner,
          metadata,
        };
      } catch {
        return null;
      }
    }),

  latest: publicProcedure
    .input(
      z.object({
        owner: z.string().optional(),
      })
    )
    .query(async ({ input }) => {
      const contract = Space__factory.connect(SPACE_ADDRESS, ethersProvider);

      const totalSupplyBigNumber = await contract.totalSupply();
      const totalSupply = totalSupplyBigNumber.toNumber();

      const spaces = [];
      let i = 0;

      while (spaces.length < 10) {
        const tokenId = totalSupply - i;
        if (tokenId <= 0) break;

        const tokenURI = await contract.tokenURI(totalSupply - i);

        if (tokenURI) {
          const owner = await contract.ownerOf(tokenId);
          if (input.owner && owner !== input.owner) continue;

          try {
            const response = await fetch(tokenURI);
            const json = await response.json();

            const metadata = ERC721MetadataSchema.parse(json);

            spaces.push({
              id: tokenId,
              owner,
              metadata,
            });
          } catch {
            // Ignore
          }
        }

        i++;
      }

      return spaces;
    }),
});
