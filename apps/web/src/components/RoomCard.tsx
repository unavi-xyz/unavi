import { useIpfsFile, useRoom } from "ceramic";

interface Props {
  streamId: string;
}

export default function RoomCard({ streamId }: Props) {
  const { room } = useRoom(streamId);
  const image = useIpfsFile(room?.image);

  if (!room) return null;

  return (
    <div className="h-36">
      <div className="w-full h-full rounded-2xl hover:cursor-pointer flex bg-neutral-100">
        <div className="w-full md:w-1/2 p-3">
          {image && (
            <img
              src={image}
              alt="room image"
              className="w-full h-full object-cover rounded-xl"
            />
          )}
        </div>

        <div className="w-full h-full px-6 flex flex-col justify-center">
          <div className="text-xl font-medium w-full">{room?.name}</div>
          <div className="text-neutral-500 w-full">{room?.description}</div>
        </div>
      </div>
    </div>
  );
}
