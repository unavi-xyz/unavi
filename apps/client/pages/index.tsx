import { useConnectModal } from "@rainbow-me/rainbowkit";
import Image from "next/future/image";
import Link from "next/link";
import { useRouter } from "next/router";
import { useContext } from "react";
import { FaBook, FaDiscord } from "react-icons/fa";
import { MdArrowDownward } from "react-icons/md";
import { VscGithubInverted, VscTwitter } from "react-icons/vsc";

import { LensContext } from "@wired-xr/lens";

import {
  DISCORD_URL,
  DOCS_URL,
  GITHUB_URL,
  TWITTER_URL,
} from "../src/constants";
import { getNavbarLayout } from "../src/home/layouts/NavbarLayout/NavbarLayout";
import MetaTags from "../src/ui/MetaTags";
import Button from "../src/ui/base/Button";

export default function Index() {
  const { handle } = useContext(LensContext);
  const { openConnectModal } = useConnectModal();
  const router = useRouter();

  function handlePlay() {
    if (!handle && openConnectModal) {
      openConnectModal();
      return;
    }

    router.push("/explore");
  }

  return (
    <>
      <MetaTags />

      <div className="flex justify-center">
        <div className="max-w mx-4 snap-mandatory snap-y">
          <div className="h-screen snap-center -mt-12 flex flex-col md:flex-row items-center">
            <div className="w-full h-full flex flex-col justify-center">
              <div className="text-8xl font-black">The Wired</div>

              <div className="text-3xl pt-2">
                A web-based virtual worlds platform <strong>done right</strong>.
              </div>

              <div className="flex justify-between md:justify-start space-x-4 pt-8 text-xl">
                <div className="w-full md:w-fit">
                  <Button
                    variant="filled"
                    squared="large"
                    fullWidth
                    onClick={handlePlay}
                  >
                    <div className="px-2 py-1">Play Now</div>
                  </Button>
                </div>

                <div className="w-full md:w-fit">
                  <Button variant="text" squared="large" fullWidth>
                    <a href={DOCS_URL} target="_blank" rel="noreferrer">
                      <div className="px-2 py-1">Learn More</div>
                    </a>
                  </Button>
                </div>
              </div>
            </div>

            <div className="relative md:h-full md:w-1/2">
              <Image
                src="/images/jump.png"
                priority
                fill
                sizes="24vw"
                alt="Wired-chan"
                className="select-none object-contain"
              />
            </div>
          </div>

          <div className="flex justify-center -mt-14">
            <MdArrowDownward
              className="animate-bounce text-5xl rounded-full p-1
                         bg-surfaceDark text-onSurfaceDark"
            />
          </div>

          <div className="h-screen snap-center flex items-center">
            <div className="md:w-1/2 md:h-1/2 md:p-8 md:mt-8">
              <div className="relative w-full h-full">
                <Image
                  src="/images/Create.png"
                  fill
                  sizes="24vw"
                  alt="Wired-chan"
                  className="select-none object-contain"
                />
              </div>
            </div>

            <div className="w-full space-y-2">
              <div
                className="text-6xl font-black rounded-xl px-4 py-2 w-fit
                         text-onPrimaryContainer bg-primaryContainer"
              >
                Create
              </div>

              <div className="-ml-1 text-5xl">
                Leave your mark on cyberspace
              </div>

              <div className="text-xl text-outline">
                Whether you{"'"}re looking to build a digital home, start a
                clothing brand, or just scratch that creative itch - The Wired
                has you covered.
              </div>

              <div className="flex justify-between md:justify-start space-x-4 pt-8 text-xl">
                <div className="w-fit">
                  <Button variant="filled" squared="large">
                    <Link href="/create" passHref>
                      <div className="px-2 py-1">Start Creating</div>
                    </Link>
                  </Button>
                </div>
              </div>
            </div>
          </div>

          <div className="h-screen snap-center flex items-center">
            <div className="w-full space-y-2">
              <div
                className="text-6xl font-black rounded-xl px-4 py-2 w-fit
                         text-onPrimaryContainer bg-primaryContainer"
              >
                Explore
              </div>

              <div className="-ml-1 text-5xl">Discover new experiences</div>

              <div className="text-xl text-outline">
                Go rock climbing on Ganymede, hit an underground music festival,
                watch the sunset on Tatooine with a group of friends - who knows
                what you
                {"'"}ll find.
              </div>

              <div className="flex justify-between md:justify-start space-x-4 pt-8 text-xl">
                <Button variant="filled" squared="large">
                  <Link href="/explore" passHref>
                    <div className="px-2 py-1">Start Exploring</div>
                  </Link>
                </Button>
              </div>
            </div>

            <div className="md:w-2/3 md:h-1/2 md:p-8 md:mt-8">
              <div className="relative w-full h-full">
                <Image
                  src="/images/Explore.png"
                  fill
                  sizes="24vw"
                  alt="Wired-chan"
                  className="select-none object-contain"
                />
              </div>
            </div>
          </div>

          <div className="h-screen snap-center flex items-center">
            <div className="md:w-1/2 md:h-1/2 md:p-8 md:mt-8">
              <div className="relative w-full h-full">
                <Image
                  src="/images/Open.png"
                  fill
                  sizes="24vw"
                  alt="Wired-chan"
                  className="select-none object-contain"
                />
              </div>
            </div>

            <div className="w-full space-y-2">
              <div
                className="text-6xl font-black rounded-xl px-4 py-2 w-fit
                         text-onPrimaryContainer bg-primaryContainer"
              >
                Open
              </div>

              <div className="-ml-1 text-5xl">
                Take control of your digital life
              </div>

              <div className="text-xl text-outline">
                Above all, The Wired is an open platform. Anyone can run their
                own game servers, modify their client, or build something new on
                top of it.
              </div>

              <div className="flex justify-between md:justify-start space-x-4 pt-8 text-xl">
                <Button variant="filled" squared="large">
                  <a href={DOCS_URL} target="_blank" rel="noreferrer">
                    <div className="px-2 py-1">Learn More</div>
                  </a>
                </Button>
              </div>
            </div>
          </div>

          <div className="h-screen snap-center flex items-center">
            <div className="md:w-1/2 md:h-1/2 md:p-8 md:mt-8">
              <div className="relative w-full h-full">
                <Image
                  src="/images/sitting.png"
                  fill
                  sizes="24vw"
                  alt="Wired-chan"
                  className="select-none object-contain"
                />
              </div>
            </div>

            <div className="w-full space-y-2">
              <div
                className="text-6xl font-black rounded-xl px-4 py-2 w-fit
                         text-onPrimaryContainer bg-primaryContainer"
              >
                Links
              </div>

              <div className="flex flex-col space-y-4 text-2xl pt-4">
                <a
                  href={DISCORD_URL}
                  target="_blank"
                  rel="noreferrer"
                  className="transition hover:ring-1 hover:ring-outline rounded-full px-4 py-0.5"
                >
                  <div className="flex items-center space-x-2">
                    <FaDiscord />
                    <div>Discord</div>
                  </div>
                </a>

                <a
                  href={TWITTER_URL}
                  target="_blank"
                  rel="noreferrer"
                  className="transition hover:ring-1 hover:ring-outline rounded-full px-4 py-0.5"
                >
                  <div className="flex items-center space-x-2">
                    <VscTwitter />
                    <div>Twitter</div>
                  </div>
                </a>

                <a
                  href={GITHUB_URL}
                  target="_blank"
                  rel="noreferrer"
                  className="transition hover:ring-1 hover:ring-outline rounded-full px-4 py-0.5"
                >
                  <div className="flex items-center space-x-2">
                    <VscGithubInverted />
                    <div>GitHub</div>
                  </div>
                </a>

                <a
                  href={DOCS_URL}
                  target="_blank"
                  rel="noreferrer"
                  className="transition hover:ring-1 hover:ring-outline rounded-full px-4 py-0.5"
                >
                  <div className="flex items-center space-x-2">
                    <FaBook />
                    <div>Docs</div>
                  </div>
                </a>
              </div>
            </div>
          </div>
        </div>
      </div>
    </>
  );
}

Index.getLayout = getNavbarLayout;
