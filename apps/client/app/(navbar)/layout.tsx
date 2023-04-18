import Image from "next/image";
import Link from "next/link";

import Logo from "@/public/images/Logo.png";
import { env } from "@/src/env.mjs";

import ClientButtons from "./ClientButtons";
import NavbarTab from "./NavbarTab";

export default function NavbarLayout({ children }: { children: React.ReactNode }) {
  return (
    <div>
      <div className="sticky top-0 z-20 h-14 w-full">
        <nav
          className="flex h-full w-full justify-center bg-white backdrop-blur-lg"
          style={{ paddingLeft: "calc(100vw - 100%)" }}
        >
          <div className="max-w-content mx-4 flex justify-between lg:grid lg:grid-cols-3">
            <Link href="/" className="flex h-full w-fit items-center">
              <div className="flex items-center space-x-2">
                <Image src={Logo} alt="logo" priority width={40} height={40} />

                <div className="hidden whitespace-nowrap text-lg font-black md:block">UNAVI</div>
              </div>
            </Link>

            <div className="flex items-center justify-center space-x-1 lg:space-x-5">
              <div>
                <NavbarTab text="Explore" href="/explore" />
              </div>

              {env.NEXT_PUBLIC_HAS_DATABASE && env.NEXT_PUBLIC_HAS_S3 ? (
                <div className="hidden lg:block">
                  <NavbarTab text="Create" href="/create" />
                </div>
              ) : null}
            </div>

            <div className="flex items-center justify-end">
              <ClientButtons />
            </div>
          </div>
        </nav>
      </div>

      <div className="h-full">{children}</div>
    </div>
  );
}
