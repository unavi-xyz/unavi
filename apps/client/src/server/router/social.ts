import { Profile__factory, PROFILE_ADDRESS } from "contracts";
import { z } from "zod";

import { ethersProvider } from "../constants";
import { getProfileFromAddress } from "../helpers/getProfileFromAddress";
import { getProfileHandle } from "../helpers/getProfileHandle";
import { getProfileMetadata } from "../helpers/getProfileMetadata";
import { getProfileOwner } from "../helpers/getProfileOwner";
import { publicProcedure, router } from "./trpc";

export const socialRouter = router({
  profile: router({
    byId: publicProcedure
      .input(
        z.object({
          id: z.number(),
        })
      )
      .query(async ({ input }) => {
        const [owner, handle, metadata] = await Promise.all([
          getProfileOwner(input.id),
          getProfileHandle(input.id),
          getProfileMetadata(input.id),
        ]);

        return {
          id: input.id,
          owner,
          handle,
          metadata,
        };
      }),

    byAddress: publicProcedure
      .input(
        z.object({
          address: z.string(),
        })
      )
      .query(async ({ input }) => {
        return await getProfileFromAddress(input.address);
      }),

    byHandle: publicProcedure
      .input(
        z.object({
          string: z.string(),
          id: z.number(),
        })
      )
      .query(async ({ input }) => {
        const contract = Profile__factory.connect(PROFILE_ADDRESS, ethersProvider);

        const idBigNumber = await contract.getProfileFromHandle(input.string, input.id);
        const id = idBigNumber.toNumber();

        const [owner, handle, metadata] = await Promise.all([
          getProfileOwner(id),
          getProfileHandle(id),
          getProfileMetadata(id),
        ]);

        return {
          id,
          owner,
          handle,
          metadata,
        };
      }),
  }),
});
