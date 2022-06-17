import { FaBook, FaDiscord } from "react-icons/fa";
import { VscGithubInverted, VscTwitter } from "react-icons/vsc";

import { getNavbarLayout } from "../src/components/layouts/NavbarLayout/NavbarLayout";
import MetaTags from "../src/components/ui/MetaTags";
import {
  DISCORD_URL,
  DOCS_URL,
  GITHUB_URL,
  TWITTER_URL,
} from "../src/helpers/constants";

export default function Index() {
  return (
    <>
      <MetaTags />

      <div className="max-w mx-auto">
        <div className="w-full grid grid-cols-3 gap-4">
          <div
            className="w-full flex flex-col justify-center items-center h-64 md:rounded-3xl
                     bg-primaryContainer text-onPrimaryContainer space-y-2 p-8 col-span-full"
          >
            <div className="text-4xl font-black text-center">
              Welcome to The Wired 👋
            </div>
          </div>

          <div className="col-span-2" />

          <div className="col-span-full md:col-span-1 px-4 py-6 space-y-4">
            <div className="text-2xl font-black text-center">🔗 Links</div>

            <div className="flex flex-col space-y-2 text-lg">
              <a
                href={DISCORD_URL}
                target="_blank"
                rel="noreferrer"
                className="hover:ring-1 hover:ring-outline rounded-full px-4 py-0.5"
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
                className="hover:ring-1 hover:ring-outline rounded-full px-4 py-0.5"
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
                className="hover:ring-1 hover:ring-outline rounded-full px-4 py-0.5"
              >
                <div className="flex items-center space-x-2">
                  <VscGithubInverted />
                  <div>GitHub</div>
                </div>
              </a>

              <a
                href={DOCS_URL}
                className="hover:ring-1 hover:ring-outline rounded-full px-4 py-0.5"
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
    </>
  );
}

Index.getLayout = getNavbarLayout;
