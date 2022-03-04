import { useRoom } from "ceramic";

interface Props {
  roomId: string;
  selected?: boolean;
}

export default function RoomCard({ roomId, selected }: Props) {
  const { room } = useRoom(roomId);

  const color = selected
    ? "bg-neutral-800 hover:bg-neutral-700 text-white"
    : "bg-neutral-200 hover:bg-neutral-300 text-black";

  const textColor = selected ? "text-neutral-300" : "text-neutral-700";

  return (
    <div
      className={`flex space-x-8 rounded-xl p-4 hover:cursor-pointer
                  transition-all duration-100 h-40 ${color}`}
    >
      <img
        src={room?.image}
        alt=""
        className="w-60 h-full object-cover rounded-lg"
      />

      <div className="flex flex-col pt-4">
        <div className="text-2xl font-medium">{room?.name}</div>
        <div className={`text-lg ${textColor}`}>{room?.description}</div>
      </div>
    </div>
  );
}
