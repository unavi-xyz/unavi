import Image from "next/image";
import Link from "next/link";
import { Suspense } from "react";

import Logo from "@/public/images/Logo.png";
import { env } from "@/src/env.mjs";

import AccountButton from "./AccountButton";
import NavbarTab from "./NavbarTab";
import SignInButton from "./SignInButton";

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
              <Image src={Logo} alt="logo" priority width={40} height={40} />
            </Link>

            <div className="flex items-center justify-center space-x-8 lg:space-x-12">
              <div>
                <NavbarTab text="Home" href="/" />
              </div>

              {env.NEXT_PUBLIC_HAS_DATABASE && env.NEXT_PUBLIC_HAS_S3 ? (
                <div className="hidden lg:block">
                  <NavbarTab text="Create" href="/create" />
                </div>
              ) : null}

              <a
                href={env.NEXT_PUBLIC_DOCS_URL}
                target="_blank"
                className="text-lg font-bold text-neutral-400"
              >
                About
              </a>
            </div>

            <div className="flex items-center justify-end">
              <Suspense fallback={<SignInButton />}>
                {/* @ts-expect-error Server Component */}
                <AccountButton />
              </Suspense>
            </div>
          </div>
        </nav>
      </div>

      <div className="h-full">{children}</div>
    </div>
  );
}
