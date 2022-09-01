import Head from "next/head";

//host url needs to be set in .env file for image meta tags to work
const rawHost = process.env.HOST ?? "";

//add http if not already there
const host = rawHost.includes("http") ? rawHost : `https://${rawHost}`;

interface Props {
  title?: string;
  description?: string;
  image?: string;
  card?: "summary" | "summary_large_image";
}

export default function MetaTags({
  title = "The Wired",
  description = "An open and decentralized 3d social platform",
  image = "/images/Logo.png",
  card = "summary",
}: Props) {
  const width = card === "summary_large_image" ? "1200" : "256";

  //if image is an external url, fetch it through next
  const localImage = image.startsWith("http")
    ? `${host}/_next/image/?url=${image}&w=${width}&q=75`
    : image;

  return (
    <Head>
      {title === "The Wired" ? (
        <title>{title}</title>
      ) : (
        <title>{`${title} / The Wired`}</title>
      )}

      <meta name="name" content={title} />
      <meta name="description" content={description} />
      <meta name="image" content={localImage} />

      {/* open graph */}
      <meta property="og:site_name" content="The Wired" />
      <meta property="og:title" content={title} />
      <meta property="og:description" content={description} />
      <meta property="og:image" content={localImage} />

      {/* twitter */}
      <meta name="twitter:card" content={card} />
      <meta name="twitter:title" content={title} />
      <meta name="twitter:description" content={description} />
      <meta name="twitter:image" content={localImage} />
      <meta name="twitter:site" content="@TheWiredXR" />

      {/* apple */}
      <meta name="apple-mobile-web-app-capable" content="yes" />
      <meta name="apple-mobile-web-app-status-bar-style" content="default" />
      <meta name="apple-mobile-web-app-title" content={title} />
    </Head>
  );
}
