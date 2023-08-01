import { World } from "@wired-protocol/types";

interface Props {
  metadata: World;
}

export default async function About({ metadata }: Props) {
  return (
    <div className="space-y-12">
      {metadata?.description && (
        <div>
          <div className="text-lg font-semibold">Description</div>

          <div className="whitespace-pre-line text-neutral-800">
            {metadata.description}
          </div>
        </div>
      )}
    </div>
  );
}
