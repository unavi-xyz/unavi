import Head from "next/head";

import { env } from "../env/server.mjs";

const origin = typeof window !== "undefined" ? window.location.origin : env.NEXTAUTH_URL;

export type PageMetadata = {
  title: string | null;
  description: string | null;
  image: string | null;
};

interface Props {
  title?: string;
  description?: string;
  image?: string;
  card?: "summary" | "summary_large_image";
}

const DEFAULT_TITLE = "Welcome to the Wired";

export default function MetaTags({
  title = DEFAULT_TITLE,
  description = "An open metaverse platform.",
  image = "/images/Hero.png",
  card = "summary_large_image",
}: Props) {
  const width = card === "summary_large_image" ? "1200" : "256";

  // If image is an external url, use the version hosted by next/image
  // this is a bit of a hack, but not sure how else to do it
  const externalImage = image
    ? image.startsWith("http")
      ? `${origin}/_next/image/?url=${image}&w=${width}&q=75`
      : image
    : null;

  return (
    <Head>
      {title === DEFAULT_TITLE ? <title>{title}</title> : <title>{`${title} / The Wired`}</title>}

      <meta name="name" content={title} />
      <meta name="description" content={description} />
      {externalImage && <meta name="image" content={externalImage} />}

      {/* open graph */}
      <meta property="og:site_name" content="The Wired" />
      <meta property="og:title" content={title} />
      <meta property="og:description" content={description} />
      {externalImage && <meta property="og:image" content={externalImage} />}

      {/* twitter */}
      <meta name="twitter:card" content={card} />
      <meta name="twitter:title" content={title} />
      <meta name="twitter:description" content={description} />
      {externalImage && <meta name="twitter:image" content={externalImage} />}
      <meta name="twitter:site" content="@wired_xr" />

      {/* apple */}
      <meta name="apple-mobile-web-app-capable" content="yes" />
      <meta name="apple-mobile-web-app-status-bar-style" content="default" />
      <meta name="apple-mobile-web-app-title" content={title} />
    </Head>
  );
}
