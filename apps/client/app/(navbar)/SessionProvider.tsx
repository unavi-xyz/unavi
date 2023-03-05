"use client";

import { SessionProvider as AuthProvider } from "next-auth/react";

interface Props {
  children: React.ReactNode;
}

export default function SessionProvider({ children }: Props) {
  return <AuthProvider>{children}</AuthProvider>;
}
