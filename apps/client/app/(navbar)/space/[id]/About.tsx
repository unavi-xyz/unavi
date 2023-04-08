import { SPACE_ADDRESS } from "contracts";

import { SpaceMetadata } from "@/src/server/helpers/readSpaceMetadata";
import { SpaceId } from "@/src/utils/parseSpaceId";

interface Props {
  id: SpaceId;
  metadata: SpaceMetadata;
}

export default async function About({ id, metadata }: Props) {
  return (
    <div className="space-y-12">
      {metadata.description && (
        <div>
          <div className="text-lg font-semibold">Description</div>

          <div className="whitespace-pre-line text-neutral-800">{metadata.description}</div>
        </div>
      )}

      {id.type === "tokenId" && (
        <div>
          <div className="text-lg font-semibold">Details</div>

          <div className="flex items-stretch text-neutral-800 md:w-fit md:space-x-10">
            <div className="flex w-full min-w-fit flex-col justify-between text-neutral-500 md:w-1/3">
              <div>Contract Address</div>
              <div>Token ID</div>
            </div>

            <div className="flex w-full flex-col justify-between">
              <a href={`https://sepolia.etherscan.io/address/${SPACE_ADDRESS}`} target="_blank">
                {SPACE_ADDRESS.slice(0, 16)}...
              </a>

              <div>{id.value}</div>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
