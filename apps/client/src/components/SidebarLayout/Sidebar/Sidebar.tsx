import { AiOutlineHome } from "react-icons/ai";
import { FaPencilRuler } from "react-icons/fa";
import { IoIosBody } from "react-icons/io";
import { BsFillPersonFill } from "react-icons/bs";
import Link from "next/link";
import Image from "next/image";

import { useLensStore } from "../../../helpers/lens/store";
import SidebarButton, { Colors } from "./SidebarButton";
import BottomButton from "./BottomButton/BottomButton";

export default function Sidebar() {
  const handle = useLensStore((state) => state.handle);

  return (
    <div className="flex flex-col w-72 bg-white">
      <div className="p-8 h-full flex flex-col justify-between">
        <div className="space-y-3">
          <div className="flex justify-center py-4">
            <Link href="/" passHref>
              <span>
                <Image
                  width={100}
                  height={100}
                  src={"/images/plug.png"}
                  alt="plug"
                />
              </span>
            </Link>
          </div>

          <SidebarButton
            icon={<AiOutlineHome />}
            text="Home"
            color={Colors.sky}
            href="/"
          />

          <SidebarButton
            icon={<IoIosBody />}
            text="Avatars"
            color={Colors.lime}
            href="/avatars"
          />

          <SidebarButton
            icon={<FaPencilRuler className="text-sm" />}
            color={Colors.amber}
            text="Editor"
            href="/editor"
          />

          <hr />

          {handle && (
            <SidebarButton
              icon={<BsFillPersonFill />}
              color={Colors.red}
              text="Profile"
              href={`/user/${handle}`}
            />
          )}
        </div>
      </div>

      <div className="h-20">
        <BottomButton />
      </div>
    </div>
  );
}
