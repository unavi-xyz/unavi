import Head from "next/head";

const origin = `https://${process.env.VERCEL_URL}`;

interface Props {
  title?: string;
  description?: string;
  image?: string | null;
  card?: "summary" | "summary_large_image";
}

export default function MetaTags({
  title = "The Wired",
  description = "A web-based virtual worlds platform done right.",
  image = "/images/Logo.png",
  card = "summary",
}: Props) {
  const width = card === "summary_large_image" ? "1200" : "256";

  // If image is an external url, fetch it through next
  const localImage = image
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
      {localImage && <meta name="image" content={localImage} />}

      {/* open graph */}
      <meta property="og:site_name" content="The Wired" />
      <meta property="og:title" content={title} />
      <meta property="og:description" content={description} />
      {localImage && <meta property="og:image" content={localImage} />}

      {/* twitter */}
      <meta name="twitter:card" content={card} />
      <meta name="twitter:title" content={title} />
      <meta name="twitter:description" content={description} />
      {localImage && <meta name="twitter:image" content={localImage} />}
      <meta name="twitter:site" content="@TheWiredXR" />

      {/* apple */}
      <meta name="apple-mobile-web-app-capable" content="yes" />
      <meta name="apple-mobile-web-app-status-bar-style" content="default" />
      <meta name="apple-mobile-web-app-title" content={title} />
    </Head>
  );
}
