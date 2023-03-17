import Head from "next/head";

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
  return (
    <Head>
      {title === DEFAULT_TITLE ? <title>{title}</title> : <title>{`${title} / The Wired`}</title>}

      <meta name="name" content={title} />
      <meta name="description" content={description} />
      {image && <meta name="image" content={image} />}

      {/* open graph */}
      <meta property="og:site_name" content="The Wired" />
      <meta property="og:title" content={title} />
      <meta property="og:description" content={description} />
      {image && <meta property="og:image" content={image} />}

      {/* twitter */}
      <meta name="twitter:card" content={card} />
      <meta name="twitter:title" content={title} />
      <meta name="twitter:description" content={description} />
      {image && <meta name="twitter:image" content={image} />}
      <meta name="twitter:site" content="@wired_xr" />

      {/* apple */}
      <meta name="apple-mobile-web-app-capable" content="yes" />
      <meta name="apple-mobile-web-app-status-bar-style" content="default" />
      <meta name="apple-mobile-web-app-title" content="The Wired" />
    </Head>
  );
}
