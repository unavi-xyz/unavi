"use client";

import { useAuth } from "@/src/client/AuthProvider";
import Button from "@/src/ui/Button";

import { useSignInStore } from "./signInStore";

export default function CreateCardButton() {
  const { status, loading } = useAuth();

  const disabled = status === "loading" || loading;

  async function handleClick() {
    if (disabled) return;

    if (status === "unauthenticated") {
      useSignInStore.setState({ open: true });
      return;
    }
  }

  return (
    <Button
      type="submit"
      onClick={handleClick}
      disabled={disabled}
      className="z-10 text-lg"
    >
      Start building
    </Button>
  );
}
