import "./globals.css";

import { Nunito } from "next/font/google";
import { lazy, Suspense } from "react";

import { baseMetadata } from "./metadata";
import Toast from "./Toast";

const font = Nunito({ subsets: ["latin"] });

export const metadata = {
  ...baseMetadata,
};

const ClientWrapper = lazy(() => import("./ClientWrapper"));

export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <html lang="en">
      <body className={font.className}>
        <ClientWrapper>{children}</ClientWrapper>

        <Suspense>
          <Toast />
        </Suspense>
      </body>
    </html>
  );
}
