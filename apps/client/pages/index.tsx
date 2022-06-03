import { FaDiscord } from "react-icons/fa";
import { VscGithubInverted, VscTwitter } from "react-icons/vsc";

import { getNavbarLayout } from "../src/components/layouts/NavbarLayout/NavbarLayout";
import MetaTags from "../src/components/ui/MetaTags";
import { DISCORD_URL, GITHUB_URL, TWITTER_URL } from "../src/helpers/constants";

export default function Index() {
  return (
    <>
      <MetaTags />

      <div className="flex justify-center py-8 mx-4">
        <div className="max-w space-y-8">
          <div
            className="flex flex-col justify-center items-center h-72 rounded-3xl
                   bg-primaryContainer text-onPrimaryContainer space-y-4"
          >
            <div className="text-4xl font-black">Welcome to The Wired 👋</div>

            <div className="flex space-x-2">
              <a href={TWITTER_URL} target="_blank" rel="noreferrer">
                <div
                  className="rounded-full hover:ring-1 hover:ring-onPrimaryContainer px-3 py-0.5
                           cursor-pointer select-none transition"
                >
                  <div className="flex items-center space-x-2">
                    <VscTwitter />
                    <div>Twitter</div>
                  </div>
                </div>
              </a>

              <a href={DISCORD_URL} target="_blank" rel="noreferrer">
                <div
                  className="rounded-full hover:ring-1 hover:ring-onPrimaryContainer px-3 py-0.5
                cursor-pointer select-none transition"
                >
                  <div className="flex items-center space-x-2">
                    <FaDiscord />
                    <div>Discord</div>
                  </div>
                </div>
              </a>

              <a href={GITHUB_URL} target="_blank" rel="noreferrer">
                <div
                  className="rounded-full hover:ring-1 hover:ring-onPrimaryContainer px-3 py-0.5
                cursor-pointer select-none transition"
                >
                  <div className="flex items-center space-x-2">
                    <VscGithubInverted />
                    <div>GitHub</div>
                  </div>
                </div>
              </a>
            </div>
          </div>

          <div className="h-full flex space-x-8">
            <div className="h-full w-full p-8 space-y-4">
              <div className="flex justify-center text-2xl font-black"></div>
              <div className="text-lg"></div>
            </div>

            <div className="w-full"></div>
          </div>
        </div>
      </div>
    </>
  );
}

Index.getLayout = getNavbarLayout;
