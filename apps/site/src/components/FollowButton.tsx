import { useContext, useEffect, useState } from "react";
import { LoadingButton } from "@mui/lab";
import {
  CeramicContext,
  follow,
  unfollow,
  useFollowing,
  ceramic,
} from "ceramic";

interface Props {
  id: string;
}

export default function FollowButton({ id }: Props) {
  const { userId, authenticated } = useContext(CeramicContext);

  const following = useFollowing(userId);

  const [loading, setLoading] = useState(false);
  const [isFollowing, setIsFollowing] = useState(false);
  const [hovering, setHovering] = useState(false);

  useEffect(() => {
    if (!following) return;
    const match = Object.values(following).some((item) => item === id);
    setIsFollowing(match);
  }, [following, id]);

  async function toggleFollow() {
    setLoading(true);
    if (isFollowing) await unfollow(id, ceramic);
    else await follow(id, ceramic);
    setIsFollowing(!isFollowing);
    setHovering(false);
    setLoading(false);
  }

  if (!authenticated) return null;

  return (
    <LoadingButton
      loading={loading}
      disabled={!authenticated}
      variant={isFollowing ? "outlined" : "contained"}
      onClick={toggleFollow}
      onMouseOver={() => setHovering(true)}
      onMouseOut={() => setHovering(false)}
      sx={{ width: "120px" }}
    >
      {isFollowing ? (hovering ? "Unfollow" : "Following") : "Follow"}
    </LoadingButton>
  );
}
