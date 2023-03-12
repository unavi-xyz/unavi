import "../styles/globals.css";

import { Metadata } from "next";
import { Nunito } from "next/font/google";
import { Suspense } from "react";

import Toast from "./Toast";

const font = Nunito({ subsets: ["latin"] });

const APP_NAME = "The Wired";
const TITLE = { default: APP_NAME, template: `%s / ${APP_NAME}` };
const DESCRIPTION = "An open metaverse platform";
const HERO = "/images/Hero.png";

export const metadata: Metadata = {
  applicationName: APP_NAME,
  colorScheme: "light",
  description: DESCRIPTION,
  keywords: ["Metaverse", "WebXR", "Web3"],
  manifest: "/manifest.json",
  themeColor: "#191919",
  title: TITLE,
  formatDetection: {
    telephone: false,
  },
  icons: {
    shortcut: "/images/Icon-512.png",
    icon: "/images/Icon-512.png",
    apple: [
      {
        url: "/images/Icon-192.png",
        sizes: "192x192",
      },
    ],
  },
  openGraph: {
    description: DESCRIPTION,
    siteName: APP_NAME,
    title: TITLE,
    type: "website",
    url: "https://thewired.space",
    images: [
      {
        url: HERO,
        width: 888,
        height: 500,
      },
    ],
  },
  twitter: {
    card: "summary_large_image",
    description: DESCRIPTION,
    images: [HERO],
    site: "@wired_xr",
    title: TITLE,
  },
  appleWebApp: {
    capable: true,
    title: APP_NAME,
    startupImage: {
      url: "/images/Icon-512.png",
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
