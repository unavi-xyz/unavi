import "../styles/globals.css";

import { Metadata } from "next";
import { Nunito } from "next/font/google";
import { Suspense } from "react";

import Toast from "./Toast";

const font = Nunito({ subsets: ["latin"] });

const title = {
  default: "The Wired",
  template: "%s / The Wired",
};

const description = "An open metaverse platform";
const hero = "/images/Hero.png";

export const metadata: Metadata = {
  title,
  description,
  keywords: ["Metaverse", "WebXR", "Web3"],
  themeColor: "#191919",
  colorScheme: "light",
  manifest: "/manifest.json",
  icons: {
    icon: "/images/Icon-512.png",
  },
  openGraph: {
    title,
    description,
    url: "https://thewired.space",
    siteName: "The Wired",
    images: [
      {
        url: hero,
        width: 888,
        height: 500,
      },
    ],
    type: "website",
  },
  twitter: {
    card: "summary_large_image",
    title,
    description,
    site: "@wired_xr",
    images: [hero],
  },
  appleWebApp: {
    title: "The Wired",
    capable: true,
    startupImage: {
      url: "/images/Icon-192.png",
    },
  },
};

export default function RootLayout({ children }: { children: React.ReactNode }) {
  return (
    <html lang="en">
      <body className={font.className}>
        {children}

        <Suspense>
          <Toast />
        </Suspense>
      </body>
    </html>
  );
}
