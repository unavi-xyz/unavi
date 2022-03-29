import { AiOutlineHome } from "react-icons/ai";
import { BsFillPersonFill } from "react-icons/bs";
import { FaPencilRuler } from "react-icons/fa";
import { IoIosBody } from "react-icons/io";
import Link from "next/link";
import Image from "next/image";
import { useAuth } from "ceramic";

import SidebarButton, { Colors } from "./SidebarButton";
import SignInButton from "./SignInButton/SignInButton";
import IPFSLoadingIndicator from "./IPFSLoadingIndictor";

export default function Sidebar() {
  const { authenticated, viewerId } = useAuth();

  return (
    <div className="p-8 bg-white w-64 h-full flex flex-col justify-between">
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

        {authenticated ? (
          <SidebarButton
            icon={<BsFillPersonFill />}
            color={Colors.red}
            text="Profile"
            href={`/user/${viewerId}`}
          />
        ) : (
          <SignInButton />
        )}
      </div>

      <div>
        <IPFSLoadingIndicator />
      </div>
    </div>
  );
}
