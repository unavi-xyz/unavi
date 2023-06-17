import { BsDiscord, BsGithub, BsTwitter } from "react-icons/bs";

const GITHUB_URL = "https://github.com/unavi-xyz/unavi";
const DISCORD_URL = "https://discord.gg/cazUfCCgHJ";
const TWITTER_URL = "https://twitter.com/unavi_xyz";

export default function CommunityCard() {
  return (
    <section className="relative w-full overflow-hidden rounded-3xl bg-gradient-to-tr from-indigo-300 to-sky-200 p-8">
      <div className="absolute -bottom-2 -left-6 select-none text-8xl opacity-50 md:text-9xl">
        🎉
      </div>

      <div className="flex h-full flex-col items-center justify-center space-y-2">
        <h2 className="z-10 text-center text-3xl font-bold">
          Join the community!
        </h2>

        <ul className="z-10 flex space-x-2">
          <a
            title="GitHub"
            href={GITHUB_URL}
            target="_blank"
            rel="noopener noreferrer"
            className="rounded-full p-2 text-3xl hover:opacity-80"
          >
            <BsGithub />
          </a>

          <a
            title="Discord"
            href={DISCORD_URL}
            target="_blank"
            rel="noopener noreferrer"
            className="rounded-full p-2 text-3xl hover:opacity-80"
          >
            <BsDiscord />
          </a>

          <a
            title="Twitter"
            href={TWITTER_URL}
            target="_blank"
            rel="noopener noreferrer"
            className="rounded-full p-2 text-3xl hover:opacity-80"
          >
            <BsTwitter />
          </a>
        </ul>
      </div>
    </section>
  );
}
