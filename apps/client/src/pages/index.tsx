import Image from "next/image";
import Link from "next/link";
import { FaBook, FaDiscord } from "react-icons/fa";
import { MdArrowDownward } from "react-icons/md";
import { VscGithubInverted, VscTwitter } from "react-icons/vsc";

import { DISCORD_URL, DOCS_URL, GITHUB_URL, TWITTER_URL } from "../constants";
import LandingInfoBlock from "../home/LandingInfoBlock";
import { getNavbarLayout } from "../home/layouts/NavbarLayout/NavbarLayout";
import MetaTags from "../home/MetaTags";
import Button from "../ui/Button";

export default function Index() {
  return (
    <>
      <MetaTags />

      <div className="flex justify-center pt-64">
        <div className="max-w-content mx-4 snap-y snap-mandatory space-y-12">
          <div className="h-screen snap-center pt-14 pb-8">
            <div className="flex h-full flex-col-reverse md:flex-row md:items-center">
              <div className="flex h-full w-full flex-col space-y-4 md:justify-center">
                <div className="text-center text-6xl font-black md:text-left md:text-8xl">
                  The Wired
                </div>

                <div className="pb-2 text-center text-xl md:text-left md:text-3xl">
                  An <strong>open and decentralized</strong> web-based metaverse
                  platform.
                </div>

                <div className="flex flex-col justify-between space-y-2 text-lg md:flex-row md:justify-start md:space-y-0 md:space-x-4 md:text-xl">
                  <div className="w-full md:w-fit">
                    <Link href="/explore" passHref>
                      <Button variant="filled" rounded="large" fullWidth>
                        <div className="md:py-0.5 md:px-4">Play Now</div>
                      </Button>
                    </Link>
                  </div>

                  <div className="w-full md:w-fit">
                    <Button variant="text" rounded="large" fullWidth>
                      <a href={DOCS_URL} target="_blank" rel="noreferrer">
                        <div className="md:py-0.5 md:px-4">Read Docs</div>
                      </a>
                    </Button>
                  </div>
                </div>
              </div>

              <div className="h-full w-full py-6 md:h-1/2 md:w-1/2 md:py-0">
                <div className="relative h-full w-full">
                  <Image
                    src="/images/jump.png"
                    priority
                    fill
                    sizes="341px"
                    alt="Wired-chan"
                    className="select-none object-contain"
                  />
                </div>
              </div>
            </div>

            <div className="-mt-5 flex justify-center md:-mt-8">
              <MdArrowDownward className="animate-bounce rounded-full bg-surfaceDark p-1 text-4xl text-onSurfaceDark md:text-5xl" />
            </div>
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
            body="Go rock climbing on Ganymede, hit an underground music festival, watch the sunset with a group of friends - who knows what you'll find."
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
            buttonText="Read Docs"
            buttonLink={DOCS_URL}
          />

          <div className="h-screen snap-center py-16">
            <div className="flex h-full flex-col md:flex-row md:items-center">
              <div className="h-full w-full py-4 md:h-1/2 md:w-1/2 md:p-8 md:py-0">
                <div className="relative h-full w-full">
                  <Image
                    src="/images/sitting.png"
                    fill
                    sizes="293px"
                    alt="Wired-chan"
                    className="select-none object-contain"
                  />
                </div>
              </div>

              <div className="w-full space-y-2">
                <div
                  className="w-fit rounded-xl bg-primaryContainer px-5 py-2 text-6xl
                         font-black text-onPrimaryContainer"
                >
                  Links
                </div>

                <div className="flex flex-col space-y-2 pt-4 text-2xl">
                  <a
                    href={DISCORD_URL}
                    target="_blank"
                    rel="noreferrer"
                    className="rounded-lg px-6 py-1 transition hover:bg-primaryContainer hover:text-onPrimaryContainer"
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
                    className="rounded-lg px-6 py-1 transition hover:bg-primaryContainer hover:text-onPrimaryContainer"
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
                    className="rounded-lg px-6 py-1 transition hover:bg-primaryContainer hover:text-onPrimaryContainer"
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
                    className="rounded-lg px-6 py-1 transition hover:bg-primaryContainer hover:text-onPrimaryContainer"
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
      </div>
    </>
  );
}

Index.getLayout = getNavbarLayout;
