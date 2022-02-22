import { useContext } from "react";
import { Avatar, Divider, Skeleton, Stack, Typography } from "@mui/material";
import Link from "next/link";
import { CeramicContext, useProfile } from "ceramic";

import FollowButton from "./FollowButton";

interface Props {
  id: string;
}

export default function UserItem({ id }: Props) {
  const { userId } = useContext(CeramicContext);

  const { profile, imageUrl } = useProfile(id);

  return (
    <div>
      <Stack
        direction="row"
        alignItems="center"
        spacing={2}
        sx={{ padding: 2 }}
      >
        <Link href={`/home/user/${id}`} passHref>
          <Avatar
            className="clickable"
            variant="rounded"
            src={imageUrl}
            sx={{ width: "3rem", height: "3rem" }}
          />
        </Link>

        <Stack sx={{ width: "100%" }}>
          {profile ? (
            <Link href={`/home/user/${id}`} passHref>
              <Typography className="link">{profile?.name}</Typography>
            </Link>
          ) : (
            <Skeleton width="100px" />
          )}

          {profile ? (
            <Typography>{profile.description}</Typography>
          ) : (
            <Skeleton />
          )}
        </Stack>

        <Stack>{userId !== id && <FollowButton id={id} />} </Stack>
      </Stack>

      <Divider />
    </div>
  );
}
