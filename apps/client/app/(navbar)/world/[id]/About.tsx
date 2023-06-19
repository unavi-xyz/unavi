import { WorldMetadata } from "@wired-protocol/types";

import { SpaceId } from "@/src/utils/parseSpaceId";

interface Props {
  id: SpaceId;
  metadata: WorldMetadata;
}

export default async function About({ id, metadata }: Props) {
  return (
    <div className="space-y-12">
      {metadata.info?.description && (
        <div>
          <div className="text-lg font-semibold">Description</div>

          <div className="whitespace-pre-line text-neutral-800">
            {metadata.info.description}
          </div>
        </div>
      )}
    </div>
  );
}
