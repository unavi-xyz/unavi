import { TRPCError } from "@trpc/server";
import { Profile__factory } from "contracts";
import { z } from "zod";

import { PROFILE_ADDRESS } from "../../constants";
import { ethersProvider } from "../constants";
import { publicProcedure, router } from "./trpc";

export const socialRouter = router({
  profileByAddress: publicProcedure
    .input(
      z.object({
        address: z.string(),
      })
    )
    .query(async ({ input }) => {
      const contract = Profile__factory.connect(PROFILE_ADDRESS, ethersProvider);

      try {
        const defaultProfileBigNumber = await contract.getDefaultProfile(input.address);
        const defaultProfile = defaultProfileBigNumber.toNumber();

        // No defaultProfile found
        if (defaultProfile === 0) return null;

        // Fetch handle
        const [handleString, handleIdBigNumber] = await contract.getHandle(defaultProfileBigNumber);
        const handleId = handleIdBigNumber.toNumber();

        // No handle found
        if (handleId === 0)
          return {
            id: defaultProfile,
            owner: input.address,
            handle: null,
            handleString: null,
            handleId: null,
          };

        const handleIdString = handleId.toString().padStart(4, "0");
        const handle = `${handleString}#${handleIdString}`;

        // Fetch metadata uri
        const uri = await contract.tokenURI(defaultProfileBigNumber);

        // No uri found
        if (!uri)
          return { id: defaultProfile, owner: input.address, handle, handleString, handleId };

        try {
          // Fetch metadata
          const response = await fetch(uri);
          const data = await response.json();

          return {
            id: defaultProfile,
            owner: input.address,
            handle,
            handleString,
            handleId,
            data,
          };
        } catch {
          return {
            id: defaultProfile,
            owner: input.address,
            handle,
            handleString,
            handleId,
          };
        }
      } catch {
        return null;
      }
    }),

  profileByHandle: publicProcedure
    .input(
      z.object({
        handle: z.string(),
      })
    )
    .query(async ({ input }) => {
      const contract = Profile__factory.connect(PROFILE_ADDRESS, ethersProvider);

      const [handleString, handleIdString] = input.handle.split("#");
      if (!handleString || !handleIdString) throw new TRPCError({ code: "BAD_REQUEST" });

      const handleId = parseInt(handleIdString, 10);

      try {
        // Fetch profile id
        const profileIdBigNumber = await contract.getProfileFromHandle(handleString, handleId);
        const profileId = profileIdBigNumber.toNumber();

        // Fetch owner
        const owner = await contract.ownerOf(profileIdBigNumber);

        // Fetch metadata uri
        const uri = await contract.tokenURI(profileId);

        // No uri found
        if (!uri) return { id: profileId, owner, handle: input.handle, handleString, handleId };

        try {
          // Fetch metadata
          const response = await fetch(uri);
          const data = await response.json();

          return {
            id: profileId,
            owner,
            handle: input.handle,
            handleString,
            handleId,
            data,
          };
        } catch {
          return {
            id: profileId,
            owner,
            handle: input.handle,
            handleString,
            handleId,
          };
        }
      } catch {
        return null;
      }
    }),

  profileById: publicProcedure
    .input(
      z.object({
        id: z.number(),
      })
    )
    .query(async ({ input }) => {
      const contract = Profile__factory.connect(PROFILE_ADDRESS, ethersProvider);

      try {
        // Fetch owner
        const owner = await contract.ownerOf(input.id);

        // Fetch handle
        const [handleString, handleIdBigNumber] = await contract.getHandle(input.id);
        const handleId = handleIdBigNumber.toNumber();

        // No handle found
        if (handleId === 0)
          return {
            id: input.id,
            owner,
            handle: null,
            handleString: null,
            handleId: null,
          };

        const handleIdString = handleId.toString().padStart(4, "0");
        const handle = `${handleString}#${handleIdString}`;

        // Fetch metadata uri
        const uri = await contract.tokenURI(input.id);

        // No uri found
        if (!uri) return { id: input.id, owner, handle, handleString, handleId };

        try {
          // Fetch metadata
          const response = await fetch(uri);
          const data = await response.json();

          return {
            id: input.id,
            owner,
            handle,
            handleString,
            handleId,
            data,
          };
        } catch {
          return {
            id: input.id,
            owner,
            handle,
            handleString,
            handleId,
          };
        }
      } catch {
        return null;
      }
    }),
});
