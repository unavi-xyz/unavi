import Image from "next/image";
import Link from "next/link";

import Screenshot1 from "../../public/images/Screenshot1.png";
import Screenshot2 from "../../public/images/Screenshot2.png";
import Screenshot3 from "../../public/images/Screenshot3.png";
import Screenshot4 from "../../public/images/Screenshot4.png";
import Discord from "../../public/images/svg/Discord.svg";
import GitHub from "../../public/images/svg/GitHub.svg";
import Twitter from "../../public/images/svg/Twitter.svg";
import { DISCORD_URL, DOCS_URL, GITHUB_URL, TWITTER_URL } from "../constants";
import { useAnimateOnEnter } from "../home/hooks/useAnimateOnEnter";
import { getNavbarLayout } from "../home/layouts/NavbarLayout/NavbarLayout";
import MetaTags from "../home/MetaTags";

export default function Index() {
  useAnimateOnEnter();

  return (
    <>
      <MetaTags />

      <main className="flex h-full justify-center">
        <div className="max-w-content mx-4">
          <section className="relative -mt-24 flex h-screen flex-col-reverse items-center justify-center md:flex-row md:justify-start">
            <div className="w-full space-y-2">
              <div className="text-center text-6xl font-black transition md:text-start md:text-7xl">
                The Wired
              </div>

              <div className="text-center text-xl text-neutral-900 md:text-start md:text-3xl">
                An open metaverse platform.
              </div>

              <div className="flex w-full flex-col space-y-4 pt-4 md:flex-row md:space-y-0 md:space-x-4">
                <div className="w-full md:w-fit">
                  <Link
                    href="/play/0x0d"
                    className="flex h-12 items-center justify-center rounded-full bg-neutral-900 px-10 text-2xl font-bold text-white outline-neutral-400 transition hover:scale-105 active:opacity-90"
                  >
                    Enter
                  </Link>
                </div>

                <div className="w-full md:w-fit">
                  <a
                    href={DOCS_URL}
                    target="_blank"
                    rel="noreferrer"
                    className="flex h-12 items-center justify-center rounded-full px-10 text-2xl font-bold text-neutral-900 ring-neutral-900 transition hover:ring-2 active:bg-neutral-200"
                  >
                    Learn More
                  </a>
                </div>
              </div>
            </div>

            <div className="show-on-scroll flex w-full flex-col items-end justify-center pb-24 md:pb-0">
              <div className="z-10 pr-36">
                <Image
                  src={Screenshot2}
                  alt="Screenshot of the Wired"
                  priority
                  className="rounded-3xl"
                />
              </div>
              <div className="-mt-28 pl-36">
                <Image
                  src={Screenshot1}
                  alt="Screenshot of the Wired"
                  priority
                  className="rounded-3xl"
                />
              </div>
            </div>

            <div className="absolute bottom-0 right-0 flex items-center space-x-4">
              <a
                href={GITHUB_URL}
                target="_blank"
                rel="noreferrer"
                className="transition hover:opacity-70"
              >
                <Image src={GitHub} width={35} height={35} alt="GitHub logo" />
              </a>
              <a
                href={DISCORD_URL}
                target="_blank"
                rel="noreferrer"
                className="transition hover:opacity-70"
              >
                <Image src={Discord} width={36} height={36} alt="Discord logo" />
              </a>
              <a
                href={TWITTER_URL}
                target="_blank"
                rel="noreferrer"
                className="transition hover:opacity-70"
              >
                <Image src={Twitter} width={36} height={36} alt="Twitter logo" />
              </a>
            </div>
          </section>

          <section className="show-on-scroll mt-32 flex flex-col space-y-8 md:flex-row md:space-y-0  md:space-x-20">
            <div className="w-full">
              <div className="relative h-48 w-full md:h-80">
                <Image
                  src={Screenshot3}
                  alt="Screenshot of the Wired"
                  fill
                  sizes="728"
                  className="rounded-3xl object-cover"
                />
              </div>
            </div>

            <div className="flex w-full flex-col justify-center space-y-4">
              <div className="text-center text-3xl font-black md:text-start md:text-4xl">
                The Spatial Web
              </div>
              <div className="text-center text-lg text-neutral-600 md:text-start md:text-xl">
                The Wired reimagines the web as an interconnected network of 3D spaces, instead of
                2D websites.{" "}
                <strong className="text-neutral-900">
                  Anyone can host their own space, on their own servers.
                </strong>
              </div>
            </div>
          </section>

          <section className="show-on-scroll mt-40 flex flex-col-reverse space-y-8 md:flex-row md:space-y-0 md:space-x-20">
            <div className="flex w-full flex-col justify-center space-y-4 pt-8 md:pt-0">
              <div className="text-center text-3xl font-black md:text-start md:text-4xl">
                Cross Platform
              </div>
              <div className="text-center text-lg text-neutral-600 md:text-start md:text-xl">
                All you need is a <strong className="text-neutral-900">web browser</strong> -
                explore the Wired from your phone, laptop, or VR headset. Wherever you are, you can
                stay connected with your friends.
              </div>
            </div>

            <div className="w-full">
              <div className="relative h-48 w-full md:h-80">
                <Image
                  src={Screenshot4}
                  alt="Screenshot of the Wired"
                  fill
                  sizes="728"
                  className="rounded-3xl object-cover"
                />
              </div>
            </div>
          </section>

          <section className="show-on-scroll mt-40 space-y-4 rounded-3xl p-8 outline outline-2 outline-black md:p-12">
            <div className="text-center text-3xl font-black md:text-4xl">Open by Design</div>

            <div className="mx-auto text-center text-lg text-neutral-600 md:w-1/2 md:text-xl">
              Each layer is open for others to build on. Mod your client, run your own servers - the
              possibilities are endless.
            </div>

            <div className="flex w-full justify-center">
              <div className="w-full md:w-fit">
                <a
                  href={DOCS_URL}
                  target="_blank"
                  rel="noreferrer"
                  className="flex h-11 items-center justify-center rounded-full bg-neutral-900 px-8 text-lg font-bold text-white outline-neutral-400 transition hover:scale-105 active:opacity-90"
                >
                  View Docs
                </a>
              </div>
            </div>
          </section>

          <section className="show-on-scroll mt-40 space-y-4 pb-40">
            <div className="text-center text-3xl font-black md:text-4xl">Join the Community</div>

            <div className="flex w-full flex-col justify-center space-y-4 pt-2 text-xl md:flex-row md:space-y-0 md:space-x-4">
              <div className="w-full md:w-fit">
                <a
                  href={DISCORD_URL}
                  target="_blank"
                  rel="noreferrer"
                  className="flex h-12 items-center justify-center space-x-3 rounded-xl px-6 text-xl font-bold ring-1 ring-neutral-700 transition hover:bg-neutral-200 active:opacity-80"
                >
                  <div className="relative h-8 w-8">
                    <Image src={Discord} fill alt="Discord logo" />
                  </div>
                  <div>Discord</div>
                </a>
              </div>

              <div className="w-full md:w-fit">
                <a
                  href={TWITTER_URL}
                  target="_blank"
                  rel="noreferrer"
                  className="flex h-12 items-center justify-center space-x-3 rounded-xl px-6 text-xl font-bold ring-1 ring-neutral-700 transition hover:bg-neutral-200 active:opacity-80"
                >
                  <div className="relative h-8 w-8">
                    <Image src={Twitter} fill alt="Twitter logo" />
                  </div>
                  <div>Twitter</div>
                </a>
              </div>
            </div>
          </section>
        </div>
      </main>
    </>
  );
}

Index.getLayout = getNavbarLayout;
