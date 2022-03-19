import { useIpfsFile, useSpace } from "ceramic";

interface Props {
  streamId: string;
}

export default function SpaceCard({ streamId }: Props) {
  const { space } = useSpace(streamId);
  const image = useIpfsFile(space?.image);

  if (!space) return null;

  return (
    <div className="h-36">
      <div className="w-full h-full rounded-2xl hover:cursor-pointer flex bg-neutral-50 border">
        <div className="w-full md:w-1/2 p-3">
          {image && (
            <img
              src={image}
              alt="space image"
              className="w-full h-full object-cover rounded-xl border"
            />
          )}
        </div>

        <div className="w-full h-full px-6 flex flex-col justify-center py-4">
          <div className="text-xl font-medium overflow-hidden">
            {space?.name}
          </div>
          <div className="text-neutral-500 overflow-hidden">
            {space?.description}
          </div>
        </div>
      </div>
    </div>
  );
}
