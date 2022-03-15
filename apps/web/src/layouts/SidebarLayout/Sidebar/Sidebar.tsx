import { AiOutlineHome } from "react-icons/ai";
import { BiWorld } from "react-icons/bi";
import { BsFillPersonFill } from "react-icons/bs";
import { FaPencilRuler } from "react-icons/fa";
import { useRouter } from "next/router";
import Link from "next/link";
import Image from "next/image";
import { useAuth } from "ceramic";

import SidebarButton, { Colors } from "./SidebarButton";
import SignInButton from "./SignInButton/SignInButton";
import IPFSLoadingIndicator from "./IPFSLoadingIndictor";

export default function Sidebar() {
  const router = useRouter();
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

        <Link href="/" passHref>
          <div>
            <SidebarButton
              icon={<AiOutlineHome />}
              text="Home"
              color={Colors.sky}
              selected={router.asPath === "/"}
            />
          </div>
        </Link>

        <Link href="/rooms" passHref>
          <div>
            <SidebarButton
              icon={<BiWorld />}
              color={Colors.lime}
              text="Rooms"
              selected={router.asPath === "/rooms"}
            />
          </div>
        </Link>

        <Link href="/editor" passHref>
          <div>
            <SidebarButton
              icon={<FaPencilRuler className="text-sm" />}
              color={Colors.amber}
              text="Editor"
              selected={router.asPath === "/editor"}
            />
          </div>
        </Link>

        <hr />

        {authenticated ? (
          <Link href={`/user/${viewerId}`} passHref>
            <div>
              <SidebarButton
                icon={<BsFillPersonFill />}
                color={Colors.red}
                text="Profile"
                selected={router.asPath === `/user/${viewerId}`}
              />
            </div>
          </Link>
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
