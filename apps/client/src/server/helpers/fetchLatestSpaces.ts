import { Space__factory, SPACE_ADDRESS } from "contracts";

import { cdnURL, S3Path } from "@/src/utils/s3Paths";

import { ethersProvider } from "../ethers";
import { prisma } from "../prisma";
import { readSpaceMetadata } from "./readSpaceMetadata";
import { validateSpaceNFT, ValidDatabaseSpace, ValidSpaceNFT } from "./validateSpaceNFT";

export async function fetchLatestSpaces(limit: number, owner?: string) {
  const [nftSpaces, databaseSpaces] = await Promise.all([
    fetchNFTSpaces(limit, owner),
    fetchDatabaseSpaces(limit, owner),
  ]);

  return [...nftSpaces, ...databaseSpaces];
}

async function fetchNFTSpaces(limit: number, owner?: string) {
  const spaceContract = Space__factory.connect(SPACE_ADDRESS, ethersProvider);

  const count = (await spaceContract.count()).toNumber();

  const spaces: ValidSpaceNFT[] = [];
  const length = Math.min(limit, count);
  let nextSpaceId = count - 1;

  const fetchSpace = async () => {
    if (nextSpaceId === 0 || spaces.length === length) return;

    const valid = await validateSpaceNFT(nextSpaceId--, owner);

    if (valid) spaces.push(valid);
    else await fetchSpace();
  };

  await Promise.all(Array.from({ length }).map(fetchSpace));

  return spaces.sort((a, b) => b.id.value - a.id.value);
}

async function fetchDatabaseSpaces(limit: number, owner?: string) {
  const spaces = await prisma.space.findMany({
    where: { owner, SpaceModel: { isNot: null }, tokenId: null },
    include: { SpaceModel: true },
    orderBy: { updatedAt: "desc" },
    take: limit,
  });

  const validSpaces: ValidDatabaseSpace[] = [];

  await Promise.all(
    spaces.map(async (space) => {
      if (!space.SpaceModel) return;

      const modelURI = cdnURL(S3Path.spaceModel(space.SpaceModel.publicId).model);
      const metadata = await readSpaceMetadata(modelURI);
      if (!metadata) return;

      validSpaces.push({ id: { type: "id", value: space.publicId }, metadata });
    })
  );

  return validSpaces;
}
