import { useAuth, useRoom } from "ceramic";
import Link from "next/link";
import { useRouter } from "next/router";

import Button from "../../../../components/Button";
import SpaceLayout from "../../../../layouts/SpaceLayout";

export default function SpaceRoom() {
  const router = useRouter();
  const roomId = router.query.room as string;

  const room = useRoom(roomId);
  const { authenticated } = useAuth();

  return (
    <div className="pr-8 flex flex-col space-y-4">
      <img
        src={room?.image}
        alt=""
        className="h-[600px] w-full object-cover rounded-md"
      />

      <div className="flex flex-col space-y-2">
        <div className="text-4xl font-medium">{room?.name}</div>
        <div className="text-xl">{room?.description}</div>

        {authenticated && (
          <div className="text-xl pt-2">
            <Link href={`/app?room=${roomId}`} passHref>
              <Button>
                <div className="px-2 py-1 text-lg">Join Room</div>
              </Button>
            </Link>
          </div>
        )}
      </div>
    </div>
  );
}

SpaceRoom.Layout = SpaceLayout;
