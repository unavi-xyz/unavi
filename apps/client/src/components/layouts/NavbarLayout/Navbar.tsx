import Image from "next/image";
import Link from "next/link";

import { useLensStore } from "../../../helpers/lens/store";

import NavbarTextButton from "./NavbarTextButton";
import LoginButton from "./LoginButton";
import ProfileButton from "./ProfileButton";

export default function Navbar() {
  const handle = useLensStore((state) => state.handle);

  return (
    <div className="bg-white w-full h-full flex justify-center border-b">
      <div className="max-w mx-8 flex items-center justify-between">
        <div className="w-full">
          <Link href="/" passHref>
            <div className="cursor-pointer">
              <Image width={32} height={32} src="/images/plug.png" alt="plug" />
            </div>
          </Link>
        </div>

        <div className="w-full flex items-center justify-center space-x-6">
          <NavbarTextButton text="Home" href="/" />
          <NavbarTextButton text="Explore" href="/explore" />
          <NavbarTextButton text="Create" href="/create" />
        </div>

        <div className="w-full flex items-center justify-end">
          {handle ? <ProfileButton /> : <LoginButton />}
        </div>
      </div>
    </div>
  );
}
