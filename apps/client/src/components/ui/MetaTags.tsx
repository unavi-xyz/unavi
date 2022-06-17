import Head from "next/head";

interface Props {
  title?: string;
  description?: string;
  image?: string;
  imageWidth?: string;
  imageHeight?: string;
  card?: "summary" | "summary_large_image";
}

export default function MetaTags({
  title = "The Wired",
  description = "A decentralized 3d social platform",
  image = "/images/Logo-Rounded.png",
  imageWidth = "256px",
  imageHeight = "256px",
  card = "summary",
}: Props) {
  return (
    <Head>
      {title === "The Wired" ? (
        <title>{title}</title>
      ) : (
        <title>{title} / The Wired</title>
      )}

      <meta name="name" content={title} />
      <meta name="description" content={description} />
      <meta name="image" content={image} />

      {/* open graph */}
      <meta property="og:site_name" content="The Wired" />
      <meta property="og:title" content={title} />
      <meta property="og:description" content={description} />
      <meta property="og:image" content={image} />
      {imageWidth && <meta property="og:image:width" content={imageWidth} />}
      {imageHeight && <meta property="og:image:height" content={imageHeight} />}

      {/* twitter */}
      <meta name="twitter:card" content={card} />
      <meta name="twitter:title" content={title} />
      <meta name="twitter:description" content={description} />
      <meta name="twitter:image" content={image} />
      <meta name="twitter:site" content="@TheWiredXR" />

      {/* apple */}
      <meta name="apple-mobile-web-app-capable" content="yes" />
      <meta name="apple-mobile-web-app-status-bar-style" content="default" />
      <meta name="apple-mobile-web-app-title" content={title} />
    </Head>
  );
}
