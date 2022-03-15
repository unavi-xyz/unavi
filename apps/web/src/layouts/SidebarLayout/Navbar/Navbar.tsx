import { useState } from "react";
import { useRouter } from "next/router";
import Link from "next/link";
import { useAuth } from "ceramic";

import SignInDialog from "../Sidebar/SignInButton/SignInDialog";
import NavbarButton from "./NavbarButton";

export default function Navbar() {
  const router = useRouter();
  const { authenticated, viewerId } = useAuth();

  const [open, setOpen] = useState(false);

  return (
    <div className="bg-white w-full h-14 flex items-center justify-between px-10">
      <Link href="/" passHref>
        <div>
          <NavbarButton text="Home" selected={router.asPath === "/"} />
        </div>
      </Link>

      <Link href="/rooms" passHref>
        <div>
          <NavbarButton text="Rooms" selected={router.asPath === "/rooms"} />
        </div>
      </Link>

      {authenticated ? (
        <Link href={`/user/${viewerId}`} passHref>
          <div>
            <NavbarButton
              text="Profile"
              selected={router.asPath === `/user/${viewerId}`}
            />
          </div>
        </Link>
      ) : (
        <div onClick={() => setOpen(true)}>
          <NavbarButton text="Sign In" selected={router.asPath === "/editor"} />
        </div>
      )}
      <SignInDialog open={open} setOpen={setOpen} />
    </div>
  );
}

function SignInButton() {}
