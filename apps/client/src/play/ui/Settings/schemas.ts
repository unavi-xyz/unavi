import { z } from "zod";

const nftSchema = z.object({
  _id: z.string(),
  collectionAddress: z.string(),
  collectionName: z.string(),
  collectionType: z.string(),
  ercType: z.string(),
  metadata: z.object({
    asset: z.string().optional(),
    assets: z
      .array(
        z.object({
          id: z.string().optional(),
          uri: z.string().optional(),
        })
      )
      .optional(),
    attributes: z
      .array(
        z.object({
          trait_type: z.string().optional(),
          value: z.any(),
        })
      )
      .optional(),
    description: z.string().optional(),
    image: z.string().optional(),
    licenses: z
      .object({
        allowedUser: z.string(),
        author: z.string().optional(),
        commercialUsage: z.string(),
        derivativeWorks: z.string().optional(),
        modification: z.string().optional(),
        redistribution: z.string().optional(),
        sell: z.string().optional(),
        sexualUsage: z.string(),
        violentUsage: z.string(),
      })
      .optional(),
    name: z.string().optional(),
  }),
  nftId: z.string(),
  nftURI: z.string().optional(),
  owner: z.string(),
  owners: z.number(),
  stats: z.object({
    likes: z.number(),
    views: z.number(),
  }),
});

export type Nft = z.infer<typeof nftSchema>;

export const nftsResponseSchema = z.object({
  currentPage: z.number().optional(),
  next: z.string().optional(),
  nfts: z.array(nftSchema),
  totalItemsCount: z.number(),
  totalPages: z.number(),
});
