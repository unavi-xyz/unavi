import { FaDiscord, FaTwitter, FaGithub } from "react-icons/fa";

import { DISCORD_URL, GITHUB_URL, TWITTER_URL } from "../src/helpers/constants";
import NavbarLayout from "../src/components/layouts/NavbarLayout/NavbarLayout";

export default function Index() {
  return (
    <div className="h-72 flex justify-center bg-secondaryContainer text-onSecondaryContainer">
      <div className="max-w space-y-3 flex flex-col justify-center">
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
  );
}

Index.Layout = NavbarLayout;
