import { useRouter } from "next/router";
import { useProfile } from "ceramic";

export default function User() {
  const router = useRouter();
  const did = router.query.did as string;

  const { profile, imageUrl } = useProfile(did);

  return (
    <div className="flex space-x-8 p-12">
      <img
        className="inline-block h-36 w-36 rounded-full object-cover"
        src={imageUrl}
        alt="profile picture"
      />

      <div className="flex flex-col space-y-1 justify-center">
        <p className="text-4xl"> {profile?.name}</p>
        <p className="text-lg text-neutral-500">{did}</p>
      </div>
    </div>
  );
}
