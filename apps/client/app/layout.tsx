import "./globals.css";

import { Nunito } from "next/font/google";
import { Suspense } from "react";

import ClientWrapper from "./ClientWrapper";
import { baseMetadata } from "./metadata";
import Toast from "./Toast";

const font = Nunito({ subsets: ["latin"] });

export const metadata = {
  ...baseMetadata,
};

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
