import { Head, Html, Main, NextScript } from "next/document";

export default function Document() {
  return (
    <Html lang="en">
      <Head>
        <link rel="manifest" href="/manifest.json" />

        {/* icons */}
        <link rel="icon" href="/images/Logo-Icon.png" />
        <link rel="shortcut icon" href="/images/Logo-Icon.png" />
        <link rel="apple-touch-icon" href="/images/Logo-Icon.png" />
      </Head>

      <body>
        <Main />
        <NextScript />
      </body>
    </Html>
  );
}
