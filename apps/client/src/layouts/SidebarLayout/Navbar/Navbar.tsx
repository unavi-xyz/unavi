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
    <>
      <SignInDialog open={open} setOpen={setOpen} />

      <div className="bg-white w-full h-14 flex items-center justify-between px-10">
        <Link href="/" passHref>
          <div>
            <NavbarButton text="Home" selected={router.asPath === "/"} />
          </div>
        </Link>

        <Link href="/editor" passHref>
          <div>
            <NavbarButton
              text="Editor"
              selected={router.asPath === "/editor"}
            />
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
            <NavbarButton text="Sign In" />
          </div>
        )}
      </div>
    </>
  );
}

function SignInButton() {}
