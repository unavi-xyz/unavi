import Image from "next/image";
import Link from "next/link";
import { useEffect, useState } from "react";

import { useLensStore } from "../../../helpers/lens/store";
import LoginButton from "./LoginButton";
import NavbarTab from "./NavbarTab";
import ProfileButton from "./ProfileButton";

export default function Navbar() {
  const handle = useLensStore((state) => state.handle);

  const [showProfileButton, setShowProfileButton] = useState(Boolean(handle));

  useEffect(() => {
    //wait for dialog close animation
    if (handle) setTimeout(() => setShowProfileButton(true), 200);
    else setShowProfileButton(false);
  }, [handle]);

  return (
    <div className="w-full h-full flex justify-center bg-surface">
      <div className="max-w mx-4 grid grid-cols-3">
        <div className="flex items-center">
          <Link href="/" passHref>
            <a className="cursor-pointer relative h-8 aspect-square">
              <Image src="/images/Logo.png" alt="logo" layout="fill" />
            </a>
          </Link>
        </div>

        <div className="flex items-center justify-center space-x-2 md:space-x-6">
          <NavbarTab text="Home" href="/" />
          <NavbarTab text="Explore" href="/explore" />
          <div className="hidden md:flex">
            <NavbarTab text="Create" href="/create" />
          </div>
        </div>

        <div className="flex items-center justify-end">
          {showProfileButton ? <ProfileButton /> : <LoginButton />}
        </div>
      </div>
    </div>
  );
}
