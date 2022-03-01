import { useRouter } from "next/router";
import { useProfile, useRooms } from "ceramic";

import RoomList from "../../components/RoomList/RoomList";
import SidebarLayout from "../../layouts/SidebarLayout";

export default function User() {
  const router = useRouter();
  const did = router.query.did as string;

  const { profile, imageUrl } = useProfile(did);
  const rooms = useRooms(did);

  return (
    <div className="p-8">
      <div className="flex space-x-8">
        <img
          className="inline-block h-36 w-36 rounded-full object-cover"
          src={imageUrl}
          alt="profile picture"
        />
        <div className="flex flex-col space-y-1 justify-center">
          <p className="text-4xl"> {profile?.name}</p>
          <p className="text-lg text-neutral-500">{did}</p>
        </div>
      </div>

      <div className="pt-8 px-32 flex">
        <div className="w-full"></div>
        <div className="w-full">
          <RoomList roomIds={rooms} />
        </div>
      </div>
    </div>
  );
}

User.Layout = SidebarLayout;
