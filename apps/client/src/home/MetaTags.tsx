import Head from "next/head";

import { getBaseUrl } from "../utils/getBaseUrl";

const origin = getBaseUrl();

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

export default function MetaTags({
  title = "The Wired",
  description = "An open and decentralized web-based metaverse platform.",
  image = "/images/Logo-Maskable.png",
  card = "summary",
}: Props) {
  const width = card === "summary_large_image" ? "1200" : "256";

  // If image is an external url, use the version hosted by next/image
  // this is a bit of a hack, but not sure how else to do it
  // we want images fetched from IPFS to be viewable on other sites without making our IPFS gateway public
  const externalImage = image
    ? image.startsWith("http")
      ? `${origin}/_next/image/?url=${image}&w=${width}&q=75`
      : image
    : null;

  return (
    <Head>
      {title === "The Wired" ? (
        <title>{title}</title>
      ) : (
        <title>{`${title} / The Wired`}</title>
      )}

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
      <meta name="twitter:site" content="@TheWiredXR" />

      {/* apple */}
      <meta name="apple-mobile-web-app-capable" content="yes" />
      <meta name="apple-mobile-web-app-status-bar-style" content="default" />
      <meta name="apple-mobile-web-app-title" content={title} />
    </Head>
  );
}
