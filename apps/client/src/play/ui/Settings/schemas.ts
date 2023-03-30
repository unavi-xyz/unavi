import { z } from "zod";

const nftSchema = z.object({
  _id: z.string(),
  nftId: z.string(),
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
        author: z.string().optional(),
        allowedUser: z.string(),
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
  nfts: z.array(nftSchema),
  next: z.string().optional(),
  currentPage: z.number().optional(),
  totalPages: z.number(),
  totalItemsCount: z.number(),
});
