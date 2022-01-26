import { useEffect, useState } from "react";
import { Grid, Stack, Typography } from "@mui/material";
import { Triplet } from "@react-three/cannon";
import { degToRad, radToDeg } from "three/src/math/MathUtils";
import { SceneObject } from "3d";

import { useStore } from "../../state/useStore";

import TripletField from "../components/fields/TripletField";
import ColorField from "../components/fields/ColorField";
import NumberField from "../components/fields/NumberField";

interface Props {
  object: SceneObject;
}

export default function Inspect({ object }: Props) {
  const usingGizmo = useStore((state) => state.usingGizmo);

  const [pos, setPos] = useState(object.position);
  const [rot, setRot] = useState(object.rotation);
  const [scale, setScale] = useState(object.scale);
  const [color, setColor] = useState(object.color);
  const [opacity, setOpacity] = useState(object.opacity);

  useEffect(() => {
    //save object values to ui
    if (usingGizmo) return;

    object.save();

    const degrees = object.rotation.map(radToDeg) as Triplet;

    setPos(object.position);
    setRot(degrees);
    setScale(object.scale);

    setColor(object.color);
    setOpacity(object.opacity);
  }, [object, usingGizmo]);

  useEffect(() => {
    //load ui values into object
    if (usingGizmo) return;

    const radians = rot.map(degToRad) as Triplet;

    object.position = pos;
    object.rotation = radians;
    object.scale = scale;

    object.color = color;
    object.opacity = opacity;

    object.load();
  }, [color, object, opacity, pos, rot, scale, usingGizmo]);

  return (
    <Stack spacing={3}>
      <Typography variant="h3">{object.name}</Typography>

      <div>
        <TripletField title="Position" value={pos} setValue={setPos} />
        <TripletField title="Rotation" step={1} value={rot} setValue={setRot} />
        <TripletField title="Scale" value={scale} setValue={setScale} />
      </div>

      <Typography variant="h4">Material</Typography>

      <div>
        <Grid container direction="row" alignItems="center">
          <Grid item xs={3}>
            <Typography>Color</Typography>
          </Grid>
          <Grid item xs={9}>
            <ColorField value={color} setValue={setColor} />
          </Grid>
        </Grid>
        <Grid container direction="row" alignItems="center">
          <Grid item xs={3}>
            <Typography>Opacity</Typography>
          </Grid>
          <Grid item xs={9}>
            <NumberField
              max={1}
              min={0}
              value={opacity}
              setValue={setOpacity}
            />
          </Grid>
        </Grid>
      </div>
    </Stack>
  );
}
