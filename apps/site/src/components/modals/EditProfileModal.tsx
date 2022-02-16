import { useContext, useEffect, useState } from "react";
import {
  Button,
  CardActionArea,
  CardMedia,
  Stack,
  TextField,
  Tooltip,
} from "@mui/material";
import { LoadingButton } from "@mui/lab";
import { BasicModal, useIdenticon } from "ui";
import { CeramicContext, ImageSources, useProfile } from "ceramic";

interface Props {
  open: boolean;
  handleClose: () => void;
}

export default function EditProfileModal({ open, handleClose }: Props) {
  const { userId } = useContext(CeramicContext);

  const { profile, imageUrl, merge } = useProfile(userId);
  const identicon = useIdenticon(userId);

  const [name, setName] = useState("");
  const [description, setDescription] = useState("");
  const [image, setImage] = useState<File>();
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    setName(profile?.name);
    setDescription(profile?.description);
  }, [profile]);

  async function handleSave() {
    setLoading(true);

    if (image) {
      //upload image to IPFS
      const body = new FormData();
      body.append("path", image, image.name);
      const res = await fetch(`https://ipfs.infura.io:5001/api/v0/add`, {
        method: "POST",
        body,
      });

      const { Hash } = await res.json();

      const testt = new Image();
      testt.src = URL.createObjectURL(image);

      testt.onload = async () => {
        //update basic profile
        const imageObject: ImageSources = {
          original: {
            src: "ipfs://" + Hash,
            height: testt.height,
            width: testt.width,
            mimeType: image.type,
            size: image.size,
          },
        };

        await merge({ image: imageObject });
      };
    }

    await merge({ name, description });

    location.reload();
    handleClose();
  }

  return (
    <BasicModal open={open} handleClose={handleClose} title="Edit Profile">
      <Stack spacing={3}>
        <Stack direction="row">
          <div>
            <input
              accept="image/*"
              style={{ display: "none" }}
              id="profile-picture-input"
              multiple
              type="file"
              onChange={(e) => setImage(e.target.files[0])}
            />

            <label htmlFor="profile-picture-input">
              <Tooltip
                title="Upload image"
                placement="right-end"
                enterDelay={500}
              >
                <CardActionArea component="span">
                  <CardMedia
                    component="img"
                    image={
                      image ? URL.createObjectURL(image) : imageUrl ?? identicon
                    }
                    style={{
                      height: "120px",
                      width: "120px",
                      border: "4px solid black",
                    }}
                  />
                </CardActionArea>
              </Tooltip>
            </label>
          </div>
        </Stack>

        <TextField
          variant="standard"
          label="Name"
          value={name}
          onChange={(e) => setName(e.target.value)}
        />

        <TextField
          variant="standard"
          label="Bio"
          value={description}
          onChange={(e) => setDescription(e.target.value)}
        />

        <Stack direction="row" spacing={1}>
          <Button
            variant="contained"
            color="info"
            sx={{ width: "100%" }}
            onClick={handleClose}
          >
            Cancel
          </Button>
          <LoadingButton
            loading={loading}
            variant="contained"
            color="secondary"
            sx={{ width: "100%" }}
            onClick={handleSave}
          >
            Save
          </LoadingButton>
        </Stack>
      </Stack>
    </BasicModal>
  );
}
