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
      <div className="w-full h-full rounded-2xl hover:cursor-pointer flex bg-neutral-100">
        <div className="w-full md:w-1/2 p-3">
          {image && (
            <img
              src={image}
              alt="space image"
              className="w-full h-full object-cover rounded-xl"
            />
          )}
        </div>

        <div className="w-full h-full px-6 flex flex-col justify-center">
          <div className="text-xl font-medium w-full">{space?.name}</div>
          <div className="text-neutral-500 w-full">{space?.description}</div>
        </div>
      </div>
    </div>
  );
}
