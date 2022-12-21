import { TRPCError } from "@trpc/server";
import { Profile__factory, PROFILE_ADDRESS } from "contracts";
import { z } from "zod";

import { numberToHexDisplay } from "../../utils/numberToHexDisplay";
import { ethersProvider } from "../constants";
import { getProfileFromAddress } from "../helpers/getProfileFromAddress";
import { getProfileHandle } from "../helpers/getProfileHandle";
import { getProfileMetadata } from "../helpers/getProfileMetadata";
import { getProfileOwner } from "../helpers/getProfileOwner";
import {
  createProfileCoverImageUploadURL,
  createProfileImageUploadURL,
  createProfileMetadataUploadURL,
} from "../s3";
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
          return await getProfileFromAddress(input.address);
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
        } catch {
          return null;
        }
      }),

    metadataUploadURL: protectedProcedure
      .input(
        z.object({
          id: z.number(),
        })
      )
      .mutation(async ({ ctx, input }) => {
        // Verify that the user is the owner of the profile
        const owner = await getProfileOwner(input.id);
        if (owner !== ctx.session.address) throw new TRPCError({ code: "UNAUTHORIZED" });

        const hexId = numberToHexDisplay(input.id);
        const url = await createProfileMetadataUploadURL(hexId);

        return url;
      }),

    imageUploadURL: protectedProcedure
      .input(
        z.object({
          id: z.number(),
        })
      )
      .mutation(async ({ ctx, input }) => {
        // Verify that the user is the owner of the profile
        const owner = await getProfileOwner(input.id);
        if (owner !== ctx.session.address) throw new TRPCError({ code: "UNAUTHORIZED" });

        const hexId = numberToHexDisplay(input.id);
        const url = await createProfileImageUploadURL(hexId);

        return url;
      }),

    coverImageUploadURL: protectedProcedure
      .input(
        z.object({
          id: z.number(),
        })
      )
      .mutation(async ({ ctx, input }) => {
        // Verify that the user is the owner of the profile
        const owner = await getProfileOwner(input.id);
        if (owner !== ctx.session.address) throw new TRPCError({ code: "UNAUTHORIZED" });

        const hexId = numberToHexDisplay(input.id);
        const url = await createProfileCoverImageUploadURL(hexId);

        return url;
      }),
  }),
});
