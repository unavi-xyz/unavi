import { useRoom } from "ceramic";
import Link from "next/link";
import { useRouter } from "next/router";

import { Button } from "../../components/base";
import SidebarLayout from "../../layouts/SidebarLayout/SidebarLayout";

export default function Room() {
  const router = useRouter();
  const id = router.query.id as string;

  const { room } = useRoom(id);

  return (
    <div className="p-8 space-y-4">
      <div>
        <div className="text-3xl">{room?.name}</div>
        <div className="text-lg text-neutral-500">{id}</div>
      </div>

      {room?.image && (
        <img src={room?.image} alt="room image" className="rounded" />
      )}

      <div>{room?.description}</div>

      <Link href={`/app?room=${id}`} passHref>
        <div className="max-w-xs">
          <Button>Join Room</Button>
        </div>
      </Link>
    </div>
  );
}

Room.Layout = SidebarLayout;
