import Image from "next/image";
import Link from "next/link";

import Logo from "../../../../public/images/Logo.png";
import { useSession } from "../../../client/auth/useSession";
import NavbarTab from "./NavbarTab";
import ProfileButton from "./ProfileButton";
import SignInButton from "./SignInButton";

export default function Navbar() {
  const { status } = useSession();

  return (
    <nav className="flex h-full w-full justify-center bg-white">
      <div className="max-w-content mx-4 flex justify-between md:grid md:grid-cols-3">
        <div className="flex items-center">
          <Link
            href="/"
            className="relative aspect-square h-9 cursor-pointer rounded-full outline-neutral-400"
          >
            <Image src={Logo} alt="logo" priority width={36} height={36} />
          </Link>
        </div>

        <div className="flex items-center justify-center space-x-1 md:space-x-4">
          <div>
            <NavbarTab text="Explore" href="/explore" />
          </div>
          <div className="hidden md:block">
            <NavbarTab text="Create" href="/create" />
          </div>
        </div>

        <div className="flex items-center justify-end">
          {status === "authenticated" ? <ProfileButton /> : <SignInButton />}
        </div>
      </div>
    </nav>
  );
}
