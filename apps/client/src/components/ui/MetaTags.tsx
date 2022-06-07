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
  description = "An open, decentralized, VR social platform",
  image = "/images/logo.png",
  imageWidth,
  imageHeight,
  card = "summary",
}: Props) {
  return (
    <Head>
      {title === "The Wired" ? (
        <title>{title}</title>
      ) : (
        <title>{title} / The Wired</title>
      )}

      {/* meta tags */}
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
    </Head>
  );
}
