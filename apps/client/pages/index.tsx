import { FaDiscord, FaTwitter, FaGithub } from "react-icons/fa";

import NavbarLayout from "../src/components/layouts/NavbarLayout/NavbarLayout";
import Chip from "../src/components/base/Chip";

export default function Index() {
  return (
    <div className="h-72 flex justify-center border-b bg-white">
      <div className="max-w space-y-3 flex flex-col justify-center">
        <div className="text-4xl font-black">Welcome to The Wired 👋</div>
        <div className="text-lg text-gray-600">
          The Wired is a decentralized, open source, browser-based, VR social
          platform
        </div>

        <div className="flex space-x-4">
          <a
            href="https://discord.gg/VCsAEneUMn"
            target="_blank"
            rel="noreferrer"
          >
            <Chip icon={<FaDiscord />} text="Discord" hoverable />
          </a>

          <a
            href="https://twitter.com/TheWiredXR"
            target="_blank"
            rel="noreferrer"
          >
            <Chip icon={<FaTwitter />} text="Twitter" hoverable />
          </a>

          <a
            href="https://github.com/wired-xr/wired"
            target="_blank"
            rel="noreferrer"
          >
            <Chip icon={<FaGithub />} text="GitHub" hoverable />
          </a>
        </div>
      </div>
    </div>
  );
}

Index.Layout = NavbarLayout;
