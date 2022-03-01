import RoomCard from "./RoomCard";

interface Props {
  selectedRoomId?: string;
  roomIds: string[];
  onRoomClick?: (roomId: string) => void;
}

export default function RoomList({
  selectedRoomId,
  roomIds,
  onRoomClick,
}: Props) {
  return (
    <div className="flex flex-col space-y-4">
      {roomIds?.map((roomId) => {
        return (
          <div
            key={roomId}
            onClick={(e) => {
              e.stopPropagation();
              if (onRoomClick) onRoomClick(roomId);
            }}
          >
            <RoomCard selected={selectedRoomId === roomId} roomId={roomId} />
          </div>
        );
      })}
    </div>
  );
}
