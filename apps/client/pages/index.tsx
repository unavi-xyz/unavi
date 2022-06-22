import Image from "next/image";
import Link from "next/link";
import { useState } from "react";
import { FaBook, FaDiscord } from "react-icons/fa";
import { MdArrowDownward } from "react-icons/md";
import { VscGithubInverted, VscTwitter } from "react-icons/vsc";

import Button from "../src/components/base/Button";
import Dialog from "../src/components/base/Dialog";
import LoginPage from "../src/components/layouts/NavbarLayout/LoginPage";
import { getNavbarLayout } from "../src/components/layouts/NavbarLayout/NavbarLayout";
import MetaTags from "../src/components/ui/MetaTags";
import {
  DISCORD_URL,
  DOCS_URL,
  GITHUB_URL,
  TWITTER_URL,
} from "../src/helpers/constants";

export default function Index() {
  const [open, setOpen] = useState(false);

  return (
    <>
      <MetaTags />

      <Dialog open={open} onClose={() => setOpen(false)}>
        <LoginPage />
      </Dialog>

      <div className="flex justify-center">
        <div className="max-w mx-4 snap-mandatory snap-y">
          <div className="h-screen snap-center -mt-12 flex flex-col md:flex-row items-center">
            <div className="w-full h-full flex flex-col justify-center">
              <div className="text-8xl font-black">The Wired</div>

              <div className="text-3xl pt-0.5">
                An open and decentralized 3d social platform.
              </div>

              <div className="flex justify-between md:justify-start space-x-4 pt-8 text-lg">
                <div className="w-full md:w-fit">
                  <Button
                    variant="filled"
                    squared="small"
                    fullWidth
                    onClick={() => setOpen(true)}
                  >
                    <div className="px-1">Get Started</div>
                  </Button>
                </div>

                <div className="w-full md:w-fit">
                  <Button variant="text" squared="small" fullWidth>
                    <a
                      href={DOCS_URL}
                      target="_blank"
                      rel="noreferrer"
                      className="px-1"
                    >
                      Learn More
                    </a>
                  </Button>
                </div>
              </div>
            </div>

            <div className="relative md:h-full md:w-1/2">
              <Image
                src="/images/jump.png"
                priority
                layout="fill"
                alt="Wired-chan"
                objectFit="contain"
                className="select-none"
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
                  layout="fill"
                  alt="Wired-chan"
                  objectFit="contain"
                  className="select-none"
                />
              </div>
            </div>

            <div className="w-full space-y-2">
              <div
                className="text-6xl font-black rounded-xl p-2 w-fit
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

              <div className="flex justify-between md:justify-start space-x-4 pt-8 text-lg">
                <div className="w-fit">
                  <Button variant="filled" squared="small">
                    <Link href="/create" passHref>
                      <a className="px-1">Start Creating</a>
                    </Link>
                  </Button>
                </div>
              </div>
            </div>
          </div>

          <div className="h-screen snap-center flex items-center">
            <div className="w-full space-y-2">
              <div
                className="text-6xl font-black rounded-xl p-2 w-fit
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

              <div className="flex justify-between md:justify-start space-x-4 pt-8 text-lg">
                <Button variant="filled" squared="small">
                  <Link href="/explore" passHref>
                    <a className="px-1">Start Exploring</a>
                  </Link>
                </Button>
              </div>
            </div>

            <div className="md:w-2/3 md:h-1/2 md:p-8 md:mt-8">
              <div className="relative w-full h-full">
                <Image
                  src="/images/Explore.png"
                  layout="fill"
                  alt="Wired-chan"
                  objectFit="contain"
                  className="select-none"
                />
              </div>
            </div>
          </div>

          <div className="h-screen snap-center flex items-center">
            <div className="md:w-1/2 md:h-1/2 md:p-8 md:mt-8">
              <div className="relative w-full h-full">
                <Image
                  src="/images/Open.png"
                  layout="fill"
                  alt="Wired-chan"
                  objectFit="contain"
                  className="select-none"
                />
              </div>
            </div>

            <div className="w-full space-y-2">
              <div
                className="text-6xl font-black rounded-xl p-2 w-fit
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

              <div className="flex justify-between md:justify-start space-x-4 pt-8 text-lg">
                <Button variant="filled" squared="small">
                  <a
                    href={DOCS_URL}
                    target="_blank"
                    rel="noreferrer"
                    className="px-1"
                  >
                    Learn More
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
                  layout="fill"
                  alt="Wired-chan"
                  objectFit="contain"
                  className="select-none"
                />
              </div>
            </div>

            <div className="w-full space-y-2">
              <div
                className="text-6xl font-black rounded-xl p-2 w-fit
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
