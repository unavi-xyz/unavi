import "../styles/globals.css";

import { Nunito } from "@next/font/google";

const font = Nunito({
  subsets: ["latin"],
});

const title = {
  default: "The Wired",
  template: "%s / The Wired",
};
const description = "An open metaverse platform";
const hero = "/images/Hero.png";

export const metadata = {
  title,
  description,
  keywords: ["Metaverse", "WebXR", "Web3"],
  themeColor: "#191919",
  colorScheme: "light",
  manifest: "/manifest.json",
  icons: {
    icon: "/images/Logo.png",
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
  appleWebApps: {
    title,
  },
};

export default function RootLayout({ children }: { children: React.ReactNode }) {
  return (
    <html lang="en">
      <body className={font.className}>{children}</body>
    </html>
  );
}
