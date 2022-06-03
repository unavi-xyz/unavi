import Link from "next/link";

import Button from "../../base/Button";

export default function BrowserErrorPage() {
  return (
    <div className="space-y-4">
      <h1 className="text-2xl flex justify-center">Browser not supported 😢</h1>

      <div className="text-center">
        Currently the studio only works on browsers that support the{" "}
        <a
          href="https://caniuse.com/native-filesystem-api"
          target="_blank"
          rel="noreferrer"
          className="hover:underline hover:decoration-2 font-bold text-primary"
        >
          File System Access API
        </a>
        . Please switch to a different browser (such as Chrome) to use the
        studio.
      </div>

      <Link href="/" passHref>
        <div>
          <Button fullWidth variant="outlined">
            Return to Home
          </Button>
        </div>
      </Link>
    </div>
  );
}
