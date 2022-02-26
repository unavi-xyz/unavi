import { MdInfo } from "react-icons/md";
import { BsFillGearFill, BsFillPeopleFill } from "react-icons/bs";
import { useRouter } from "next/router";
import Link from "next/link";
import { useAuth, useSpace } from "ceramic";

import CreateRoomButton from "../components/CreateRoomButton/CreateRoomButton";
import RoomList from "../components/RoomList/RoomList";

export default function SpaceLayout({ children }) {
  const router = useRouter();
  const spaceId = router.query.id as string;

  const { authenticated } = useAuth();
  const space = useSpace(spaceId);

  return (
    <div className="h-full">
      <div className="bg-white h-24 flex items-center px-8 space-x-16">
        <div className="flex flex-col">
          <p className="text-sm ml-[2px]">PUBLIC SPACE</p>
          <p className="text-3xl font-medium">{space?.name}</p>
        </div>

        <div className="flex space-x-4">
          <NavbarButton icon={<MdInfo className="text-[1.4rem]" />} />
          <NavbarButton icon={<BsFillPeopleFill />} />
          <NavbarButton icon={<BsFillGearFill />} />
        </div>
      </div>

      <div className="flex h-full w-full">
        <Link href={`/space/${spaceId}`} passHref>
          <div className="w-7/12 min-w-max px-8">
            <div className="flex justify-between items-center h-20">
              <p className="text-xl font-medium">Rooms</p>

              {authenticated && <CreateRoomButton spaceId={spaceId} />}
            </div>

            <RoomList
              roomIds={Object.values(space?.rooms ?? {})}
              basePath={`/space/${spaceId}`}
            />
          </div>
        </Link>

        <div className="w-full mt-20">{children}</div>
      </div>
    </div>
  );
}

function NavbarButton({ icon = <></> }) {
  return (
    <div
      className="w-14 h-14 bg-neutral-100 rounded-lg hover:cursor-pointer
               hover:bg-neutral-200 flex justify-center items-center text-xl
                 transition-all duration-150 shadow"
    >
      {icon}
    </div>
  );
}
