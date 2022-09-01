import Image from "next/image";
import Link from "next/link";
import { useContext } from "react";

import { LensContext } from "@wired-xr/lens";

import LoginButton from "./LoginButton";
import NavbarTab from "./NavbarTab";
import ProfileButton from "./ProfileButton";

export default function Navbar() {
  const { handle } = useContext(LensContext);

  return (
    <>
      <div className="w-full h-full flex justify-center bg-surface">
        <div className="max-w mx-4 grid grid-cols-3">
          <div className="flex items-center">
            <Link href="/" passHref>
              <a className="cursor-pointer relative h-9 aspect-square">
                <Image
                  src="/images/Logo-Maskable.png"
                  alt="logo"
                  layout="fill"
                />
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
            {handle ? <ProfileButton /> : <LoginButton />}
          </div>
        </div>
      </div>
    </>
  );
}
