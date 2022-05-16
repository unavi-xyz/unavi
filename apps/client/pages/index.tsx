import { FaDiscord, FaGithub, FaTwitter } from "react-icons/fa";

import NavbarLayout from "../src/components/layouts/NavbarLayout/NavbarLayout";
import { DISCORD_URL, GITHUB_URL, TWITTER_URL } from "../src/helpers/constants";

export default function Index() {
  return (
    <div className="flex justify-center py-8 mx-8">
      <div className="max-w space-y-8">
        <div
          className="flex flex-col justify-center items-center h-72 rounded-3xl
                   bg-primaryContainer text-onPrimaryContainer"
        >
          <div className="text-4xl font-black">Welcome to The Wired 👋</div>
        </div>

        <div></div>
      </div>
    </div>
  );

  return (
    <div className="max-w mx-auto pt-8">
      <div className="h-72 flex justify-center rounded-3xl bg-secondaryContainer text-onSecondaryContainer">
        <div className="space-y-3 flex flex-col justify-center">
          <div className="text-4xl font-black">Welcome to The Wired 👋</div>
          <div className="text-lg">
            The Wired is a decentralized, open source, browser-based, VR social
            platform
          </div>

          <div className="flex space-x-4">
            <a
              href={DISCORD_URL}
              target="_blank"
              rel="noreferrer"
              className="flex items-center space-x-2 transition rounded-full
            hover:ring-1 hover:ring-outline px-3 py-0.5"
            >
              <FaDiscord />
              <div>Discord</div>
            </a>

            <a
              href={TWITTER_URL}
              target="_blank"
              rel="noreferrer"
              className="flex items-center space-x-2 transition rounded-full
            hover:ring-1 hover:ring-outline px-3 py-0.5"
            >
              <FaTwitter />
              <div>Twitter</div>
            </a>

            <a
              href={GITHUB_URL}
              target="_blank"
              rel="noreferrer"
              className="flex items-center space-x-2 transition rounded-full
            hover:ring-1 hover:ring-outline px-3 py-0.5"
            >
              <FaGithub />
              <div>Github</div>
            </a>
          </div>
        </div>
      </div>

      <div></div>
    </div>
  );
}

Index.Layout = NavbarLayout;
