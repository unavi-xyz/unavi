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
      <div className="max-w mx-8 flex items-center justify-between">
        <div className="w-full">
          <div className="w-fit">
            <Link href="/" passHref>
              <div className="cursor-pointer">
                <Image
                  width={32}
                  height={32}
                  src="/images/plug.png"
                  alt="logo"
                  priority
                />
              </div>
            </Link>
          </div>
        </div>

        <div className="w-full flex items-center justify-center space-x-6">
          <NavbarTab text="Home" href="/" />
          <NavbarTab text="Explore" href="/explore" />
          <NavbarTab text="Create" href="/create" />
        </div>

        <div className="w-full flex items-center justify-end">
          {showProfileButton ? <ProfileButton /> : <LoginButton />}
        </div>
      </div>
    </div>
  );
}
