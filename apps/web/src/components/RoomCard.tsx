import { useRoom } from "ceramic";
import Card from "./base/Card";

interface Props {
  streamId: string;
}

export default function RoomCard({ streamId }: Props) {
  const { room } = useRoom(streamId);

  if (!room) return null;

  return (
    <div className="h-40">
      <Card text={room?.name} image={room?.image} />
    </div>
  );
}
