import Image from "next/image";
import Link from "next/link";

import { DISCORD_URL, DOCS_URL, GITHUB_URL, TWITTER_URL } from "../constants";
import { useAnimateOnEnter } from "../home/hooks/useAnimateOnEnter";
import { getNavbarLayout } from "../home/layouts/NavbarLayout/NavbarLayout";
import MetaTags from "../home/MetaTags";
import Button from "../ui/Button";

export default function Index() {
  useAnimateOnEnter();

  return (
    <>
      <MetaTags />

      <div className="flex h-full justify-center">
        <div className="max-w-content mx-4">
          <section className="show-on-scroll flex flex-col justify-center space-y-1 py-[200px] md:py-[300px]">
            <div
              className="text-center text-6xl font-black transition
            md:text-7xl"
            >
              The Wired
            </div>

            <div className="pb-2 text-center text-xl md:text-2xl">
              An <strong>open and decentralized</strong> web-based metaverse platform.
            </div>

            <div className="flex w-full flex-col justify-center space-y-4 pt-2 text-xl md:flex-row md:space-y-0 md:space-x-4">
              <div className="w-full md:w-fit">
                <Link href="/app/0x04">
                  <Button variant="filled" rounded="large" fullWidth>
                    <div className="flex h-9 items-center justify-center md:px-4">Play Now</div>
                  </Button>
                </Link>
              </div>

              <div className="w-full md:w-fit">
                <a href={GITHUB_URL} target="_blank" rel="noreferrer">
                  <Button variant="outlined" rounded="large" fullWidth>
                    <div className="flex h-9 items-center justify-center space-x-4 px-1">
                      <Image src="/images/GitHub.svg" width={32} height={32} alt="" />
                      <div>GitHub</div>
                    </div>
                  </Button>
                </a>
              </div>
            </div>
          </section>

          <section className="show-on-scroll mb-[100px] flex flex-col space-y-8 md:mb-[150px] md:flex-row md:space-y-0 md:space-x-4">
            <div className="relative h-[200px] w-full md:h-[300px]">
              <Image
                src="/images/Screenshot1.png"
                alt=""
                fill
                sizes="512"
                loading="eager"
                className="w-full rounded-3xl object-cover"
              />
            </div>

            <div className="flex w-full flex-col justify-center space-y-4">
              <div className="text-center text-3xl font-black md:text-4xl">The Spatial Web</div>

              <div className="mx-auto text-center text-lg text-neutral-500 md:text-xl">
                The Wired reimagines the web as an interconnected network of 3D spaces, instead of
                2D websites.
              </div>
            </div>
          </section>

          <section className="show-on-scroll mb-[100px] flex flex-col-reverse md:mb-[150px] md:flex-row md:space-y-0 md:space-x-4">
            <div className="flex w-full flex-col justify-center space-y-4 pt-8 md:pt-0">
              <div className="text-center text-3xl font-black md:text-4xl">Cross Platform</div>

              <div className="mx-auto text-center text-lg text-neutral-500 md:text-xl">
                All you need is a web browser - explore from your phone, laptop, or VR headset.
              </div>
            </div>

            <div className="relative h-[200px] w-full md:h-[300px]">
              <Image
                src="/images/Screenshot2.png"
                alt=""
                fill
                sizes="512"
                loading="eager"
                className="w-full rounded-3xl object-cover"
              />
            </div>
          </section>

          <section className="show-on-scroll mb-[100px] space-y-4 rounded-3xl p-8 outline outline-2 outline-black md:mb-[150px] md:p-12">
            <div className="text-center text-3xl font-black md:text-4xl">Open by Design</div>

            <div className="mx-auto text-center text-lg text-neutral-500 md:w-2/3 md:text-xl">
              Each layer is open for others to build on. Mod your client. Run your own servers. The
              possibilities are endless.
            </div>

            <div className="flex w-full justify-center">
              <div className="w-full md:w-fit">
                <a href={DOCS_URL} target="_blank" rel="noreferrer">
                  <Button variant="filled" rounded="large" fullWidth>
                    <div className="flex h-9 items-center justify-center text-xl md:px-4">
                      View Docs
                    </div>
                  </Button>
                </a>
              </div>
            </div>
          </section>

          <section className="show-on-scroll space-y-4 pb-[150px]">
            <div className="text-center text-3xl font-black md:text-4xl">Join the Community</div>

            <div className="flex w-full flex-col justify-center space-y-4 pt-2 text-xl md:flex-row md:space-y-0 md:space-x-4">
              <div className="w-full md:w-fit">
                <a href={DISCORD_URL} target="_blank" rel="noreferrer">
                  <Button variant="outlined" rounded="large" fullWidth>
                    <div className="flex h-9 items-center justify-center space-x-4 px-1">
                      <Image src="/images/Discord.svg" width={32} height={32} alt="" />
                      <div>Discord</div>
                    </div>
                  </Button>
                </a>
              </div>

              <div className="w-full md:w-fit">
                <a href={TWITTER_URL} target="_blank" rel="noreferrer">
                  <Button variant="outlined" rounded="large" fullWidth>
                    <div className="flex h-9 items-center justify-center space-x-4 px-1">
                      <Image src="/images/Twitter.svg" width={32} height={32} alt="" />
                      <div>Twitter</div>
                    </div>
                  </Button>
                </a>
              </div>
            </div>
          </section>
        </div>
      </div>
    </>
  );
}

Index.getLayout = getNavbarLayout;
