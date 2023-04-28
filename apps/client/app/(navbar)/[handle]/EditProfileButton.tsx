"use client";

import { useRouter } from "next/navigation";
import { useState } from "react";
import { toast } from "react-hot-toast";
import { MdEdit } from "react-icons/md";

import {
  MAX_PROFILE_BIO_LENGTH,
  MAX_USERNAME_LENGTH,
  MIN_USERNAME_LENGTH,
} from "@/app/api/auth/profile/constants";
import { updateProfile } from "@/app/api/auth/profile/helper";
import { parseError } from "@/src/editor/utils/parseError";
import { env } from "@/src/env.mjs";
import Button from "@/src/ui/Button";
import DialogContent, { DialogRoot, DialogTrigger } from "@/src/ui/Dialog";

interface Props {
  username: string;
  bio?: string;
}

export default function EditProfileButton({ username, bio }: Props) {
  const [open, setOpen] = useState(false);
  const [loading, setLoading] = useState(false);

  const router = useRouter();

  async function handleSubmit(e: React.FormEvent<HTMLFormElement>) {
    e.preventDefault();

    if (loading) return;

    const form = e.currentTarget;
    const usernameElement = form.elements[0] as HTMLInputElement;
    const bioElement = form.elements[1] as HTMLTextAreaElement;

    setLoading(true);

    try {
      await updateProfile({ username: usernameElement.value || username, bio: bioElement.value });

      // Refresh the page
      router.refresh();

      // Redirect to the new username
      router.push(`/@${usernameElement.value || username}`);

      // Close the dialog
      setOpen(false);
    } catch (e) {
      console.error(e);
      const message = parseError(e);
      toast.error(message);
    }

    setLoading(false);
  }

  return (
    <DialogRoot open={open} onOpenChange={setOpen}>
      <DialogContent title="Edit Profile">
        <form onSubmit={handleSubmit} className="space-y-4">
          <label title="Username" className="block space-y-1">
            <div className="font-bold text-neutral-700">Username</div>

            <div className="flex rounded-lg border border-neutral-200 bg-neutral-100">
              <div className="px-4 py-2 font-bold">
                {new URL(env.NEXT_PUBLIC_DEPLOYED_URL).host}/@
              </div>
              <input
                type="text"
                defaultValue={username}
                placeholder={username}
                minLength={MIN_USERNAME_LENGTH}
                maxLength={MAX_USERNAME_LENGTH}
                className="w-full rounded-r-lg bg-white px-4"
              />
            </div>
          </label>

          <label title="Bio" className="block space-y-1">
            <div className="font-bold text-neutral-700">Bio</div>

            <textarea
              placeholder="Say something about yourself..."
              defaultValue={bio}
              maxLength={MAX_PROFILE_BIO_LENGTH}
              className="max-h-64 w-full rounded-lg border border-neutral-200 p-4 py-2"
            />
          </label>

          <div className="flex justify-end">
            <Button type="submit" disabled={loading}>
              Save
            </Button>
          </div>
        </form>
      </DialogContent>

      <DialogTrigger asChild>
        <button
          disabled={loading}
          className="absolute bottom-0 right-0 z-20 rounded-full bg-black p-2 transition hover:bg-neutral-800 active:scale-95"
        >
          <MdEdit className="text-lg text-white" />
        </button>
      </DialogTrigger>
    </DialogRoot>
  );
}
