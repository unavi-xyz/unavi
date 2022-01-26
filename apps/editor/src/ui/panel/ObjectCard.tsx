import { useEffect, useState } from "react";
import {
  Card,
  CardActionArea,
  CardContent,
  Grid,
  Typography,
} from "@mui/material";
import { ASSETS, ASSET_NAMES } from "3d";

import { useScene } from "../../state/useScene";

interface Props {
  name: ASSET_NAMES;
}

export default function ObjectCard({ name }: Props) {
  const scene = useScene((state) => state.scene);
  const newObject = useScene((state) => state.newObject);

  const [count, setCount] = useState(0);

  const limit = ASSETS[name].limit;

  function handleClick() {
    if (count < limit || limit < 0) newObject(name);
  }

  useEffect(() => {
    const found = Object.values(scene).filter((item) => item.type === name);
    setCount(found.length);
  }, [name, scene]);

  return (
    <Grid item xs sx={{ minWidth: "120px" }}>
      <Card variant="outlined">
        <CardActionArea onClick={handleClick}>
          <CardContent>
            <Grid container>
              <Grid item xs={3}></Grid>

              <Grid item xs={6} container justifyContent="center">
                <Typography>{name}</Typography>
              </Grid>

              <Grid item xs={3} container justifyContent="flex-end">
                {limit > -1 && (
                  <Typography>
                    {count}/{limit}
                  </Typography>
                )}
              </Grid>
            </Grid>
          </CardContent>
        </CardActionArea>
      </Card>
    </Grid>
  );
}
