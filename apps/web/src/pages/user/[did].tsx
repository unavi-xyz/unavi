import { useIpfsFile, useProfile, useRooms } from "ceramic";
import Link from "next/link";
import { useRouter } from "next/router";

import { ProfilePicture } from "../../components/base";
import RoomCard from "../../components/RoomCard";
import SidebarLayout from "../../layouts/SidebarLayout/SidebarLayout";

export default function User() {
  const router = useRouter();
  const did = router.query.did as string;

  const { profile } = useProfile(did);
  const image = useIpfsFile(profile?.image?.original.src);
  const rooms = useRooms(did);

  return (
    <div className="p-16">
      <div className="flex items-center space-x-8">
        <div className="w-32 h-32">
          {image && <ProfilePicture src={image} />}
        </div>

        <div>
          <div className="text-3xl">{profile?.name}</div>
          <div className="text-lg text-neutral-400">{did}</div>
        </div>
      </div>

      <div className="w-full flex justify-end">
        <div className="w-full max-w-2xl space-y-4 p-4 overflow-auto h-[1000px]">
          {rooms?.map((streamId) => {
            return (
              <Link key={streamId} href={`/room/${streamId}`} passHref>
                <div className="hover:cursor-pointer hover:-translate-x-2 transition-all duration-100">
                  <RoomCard streamId={streamId} />
                </div>
              </Link>
            );
          })}
        </div>
      </div>
    </div>
  );
}

User.Layout = SidebarLayout;
