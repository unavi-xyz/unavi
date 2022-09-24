import dynamic from "next/dynamic";
import Image from "next/future/image";
import Link from "next/link";

import { useLens } from "../../../lib/lens/hooks/useLens";
import NavbarTab from "./NavbarTab";

const ProfileButton = dynamic(() => import("./ProfileButton"));
const LoginButton = dynamic(() => import("./LoginButton"));

export default function Navbar() {
  const { handle } = useLens();

  return (
    <div className="flex h-full w-full justify-center bg-surface">
      <div className="max-w-content mx-4 flex justify-between md:grid md:grid-cols-3">
        <div className="flex items-center">
          <Link href="/" passHref>
            <div className="relative aspect-square h-10 cursor-pointer">
              <Image
                src="/images/Logo-Icon.png"
                alt="logo"
                priority
                fill
                sizes="36px"
              />
            </div>
          </Link>
        </div>

        <div className="flex items-center justify-center space-x-4 md:space-x-5">
          <NavbarTab text="Home" href="/" />
          <NavbarTab text="Explore" href="/explore" />
          <div className="hidden md:flex">
            <NavbarTab text="Create" href="/create" />
          </div>
        </div>

        <div className="flex items-center justify-end">
          {handle ? <ProfileButton /> : <LoginButton />}
        </div>
      </div>
    </div>
  );
}
