import Link from "next/link";
import { useRouter } from "next/router";

import RoomCard from "./RoomCard";

interface Props {
  roomIds: string[];
  basePath: string;
}

export default function RoomList({ roomIds, basePath }: Props) {
  const router = useRouter();
  const selectedId = router.query.room as string;

  return (
    <div className="flex flex-col space-y-4">
      {roomIds?.map((roomId) => {
        const selected = roomId === selectedId;

        return (
          <Link key={roomId} href={`${basePath}/room/${roomId}`} passHref>
            <span>
              <RoomCard selected={selected} roomId={roomId} />
            </span>
          </Link>
        );
      })}
    </div>
  );
}
