import { TRPCError } from "@trpc/server";
import { Profile__factory, PROFILE_ADDRESS } from "contracts";
import { z } from "zod";

import { numberToHexDisplay } from "../../utils/numberToHexDisplay";
import { ethersProvider } from "../constants";
import { fetchProfileFromAddress } from "../helpers/fetchProfileFromAddress";
import { fetchProfileHandle } from "../helpers/fetchProfileHandle";
import { fetchProfileMetadata } from "../helpers/fetchProfileMetadata";
import { fetchProfileOwner } from "../helpers/fetchProfileOwner";
import { Profile } from "../s3/Profile";
import { protectedProcedure, publicProcedure, router } from "./trpc";

export const socialRouter = router({
  profile: router({
    byId: publicProcedure
      .input(
        z.object({
          id: z.number(),
        })
      )
      .query(async ({ input }) => {
        try {
          const [owner, handle, metadata] = await Promise.all([
            fetchProfileOwner(input.id),
            fetchProfileHandle(input.id),
            fetchProfileMetadata(input.id),
          ]);

          return {
            id: input.id,
            owner,
            handle,
            metadata,
          };
        } catch {
          return null;
        }
      }),

    byAddress: publicProcedure
      .input(
        z.object({
          address: z.string(),
        })
      )
      .query(async ({ input }) => {
        try {
          return await fetchProfileFromAddress(input.address);
        } catch {
          return null;
        }
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

        try {
          const idBigNumber = await contract.getProfileFromHandle(input.string, input.id);
          const id = idBigNumber.toNumber();

          const [owner, handle, metadata] = await Promise.all([
            fetchProfileOwner(id),
            fetchProfileHandle(id),
            fetchProfileMetadata(id),
          ]);

          return {
            id,
            owner,
            handle,
            metadata,
          };
        } catch {
          return null;
        }
      }),

    getMetadataUpload: protectedProcedure
      .input(
        z.object({
          id: z.number(),
        })
      )
      .mutation(async ({ ctx, input }) => {
        // Verify that the user is the owner of the profile
        const owner = await fetchProfileOwner(input.id);
        if (owner !== ctx.session.address) throw new TRPCError({ code: "UNAUTHORIZED" });

        const hexId = numberToHexDisplay(input.id);
        const profile = new Profile(hexId);
        const url = await profile.getUpload("metadata");

        return url;
      }),

    getImageUpload: protectedProcedure
      .input(
        z.object({
          id: z.number(),
        })
      )
      .mutation(async ({ ctx, input }) => {
        // Verify that the user is the owner of the profile
        const owner = await fetchProfileOwner(input.id);
        if (owner !== ctx.session.address) throw new TRPCError({ code: "UNAUTHORIZED" });

        const hexId = numberToHexDisplay(input.id);
        const profile = new Profile(hexId);
        const url = await profile.getUpload("image");

        return url;
      }),

    getCoverUpload: protectedProcedure
      .input(
        z.object({
          id: z.number(),
        })
      )
      .mutation(async ({ ctx, input }) => {
        // Verify that the user is the owner of the profile
        const owner = await fetchProfileOwner(input.id);
        if (owner !== ctx.session.address) throw new TRPCError({ code: "UNAUTHORIZED" });
        const hexId = numberToHexDisplay(input.id);
        const profile = new Profile(hexId);
        const url = await profile.getUpload("cover");

        return url;
      }),
  }),
});
