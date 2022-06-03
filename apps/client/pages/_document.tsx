import { Head, Html, Main, NextScript } from "next/document";

const title = "The Wired";
const description = "The Wired is an open, decentralized, VR social platform.";
const image =
  "https://thewired.space/_next/image?url=%2Fimages%2Fplug.png&w=128&q=100";

export default function Document() {
  return (
    <Html>
      <Head>
        {/* favicon */}
        <link rel="icon" href="/images/plug.png" />

        {/* fonts */}
        <link rel="preconnect" href="https://fonts.googleapis.com" />
        <link
          rel="preconnect"
          href="https://fonts.gstatic.com"
          crossOrigin=""
        />
        <link
          href="https://fonts.googleapis.com/css2?family=Lato:wght@400;700;900&display=swap"
          rel="stylesheet"
        />

        {/* meta tags */}
        <meta name="viewport" content="initial-scale=1, width=device-width" />
        <meta name="theme-color" content="#fffbfe" />

        {/* open graph */}
        <meta name="og:site_name" content="The Wired" />
        <meta name="og:title" content={title} />
        <meta name="og:description" content={description} />
        <meta name="og:image" content={image} />

        {/* twitter */}
        <meta name="twitter:site" content="TheWiredXR" />
        <meta name="twitter:title" content={title} />
        <meta name="twitter:description" content={description} />
        <meta name="twitter:image" content={image} />
      </Head>
      <body>
        <Main />
        <NextScript />
      </body>
    </Html>
  );
}
