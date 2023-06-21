import { WorldMetadata } from "@wired-protocol/types";

interface Props {
  metadata: WorldMetadata;
}

export default async function About({ metadata }: Props) {
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
