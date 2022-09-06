import Image from "next/future/image";
import Link from "next/link";
import { FaBook, FaDiscord } from "react-icons/fa";
import { MdArrowDownward } from "react-icons/md";
import { VscGithubInverted, VscTwitter } from "react-icons/vsc";

import {
  DISCORD_URL,
  DOCS_URL,
  GITHUB_URL,
  TWITTER_URL,
} from "../src/constants";
import { getNavbarLayout } from "../src/home/layouts/NavbarLayout/NavbarLayout";
import LandingInfoBlock from "../src/ui/LandingInfoBlock";
import MetaTags from "../src/ui/MetaTags";
import Button from "../src/ui/base/Button";

export default function Index() {
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
                  <Link href="/explore" passHref>
                    <div>
                      <Button variant="filled" squared="large" fullWidth>
                        <div className="px-1.5 py-0.5">Play Now</div>
                      </Button>
                    </div>
                  </Link>
                </div>

                <div className="w-full md:w-fit">
                  <Button variant="text" squared="large" fullWidth>
                    <a href={DOCS_URL} target="_blank" rel="noreferrer">
                      <div className="px-1.5 py-0.5">Learn More</div>
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

          <LandingInfoBlock
            title="Create"
            subtitle="Leave your mark on cyberspace"
            body="Whether you're looking to build a digital home, start a clothing brand, or just scratch that creative itch - the Wired has you covered."
            image="/images/Create.png"
            imageSide="left"
            buttonText="Start Creating"
            buttonLink="/create"
          />

          <LandingInfoBlock
            title="Explore"
            subtitle="Discover new experiences"
            body="Go rock climbing on Ganymede, hit an underground music festival, watch the sunset on Tatooine with a group of friends - who knows what you'll find."
            image="/images/Explore.png"
            imageSide="right"
            buttonText="Start Exploring"
            buttonLink="/explore"
          />

          <LandingInfoBlock
            title="Open"
            subtitle="Take control of your digital life"
            body="Above all, the Wired is an open platform. Anyone can run their own game servers, modify their client, or build something new on top of it."
            image="/images/Open.png"
            imageSide="left"
            buttonText="Learn More"
            buttonLink={DOCS_URL}
          />

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
                className="text-6xl font-black rounded-xl px-5 py-2 w-fit
                           text-onPrimaryContainer bg-primaryContainer"
              >
                Links
              </div>

              <div className="flex flex-col space-y-2 text-2xl pt-4">
                <a
                  href={DISCORD_URL}
                  target="_blank"
                  rel="noreferrer"
                  className="transition hover:bg-primaryContainer hover:text-onPrimaryContainer rounded-lg px-6 py-1"
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
                  className="transition hover:bg-primaryContainer hover:text-onPrimaryContainer rounded-lg px-6 py-1"
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
                  className="transition hover:bg-primaryContainer hover:text-onPrimaryContainer rounded-lg px-6 py-1"
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
                  className="transition hover:bg-primaryContainer hover:text-onPrimaryContainer rounded-lg px-6 py-1"
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
