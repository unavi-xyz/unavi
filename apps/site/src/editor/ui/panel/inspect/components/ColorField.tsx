import { Dispatch, SetStateAction, useEffect, useRef } from "react";
import { Grid, TextField, Typography } from "@mui/material";
import { ASSETS, ASSET_NAMES, PARAM_NAMES } from "3d";

interface Props {
  title: string;
  type: PARAM_NAMES;
  params: typeof ASSETS[ASSET_NAMES]["params"];
  setParams: Dispatch<SetStateAction<typeof ASSETS[ASSET_NAMES]["params"]>>;
}

export default function ColorField({ title, type, params, setParams }: Props) {
  const ref = useRef<HTMLInputElement>();

  useEffect(() => {
    const interval = setInterval(() => {
      setParams((prev) => {
        if (prev[type] === ref.current.value) return prev;

        const newParams = { ...prev };
        newParams[type] = ref.current.value;
        return newParams;
      });
    }, 100);

    return () => {
      clearInterval(interval);
    };
  }, [setParams, type]);

  return (
    <Grid container direction="row" alignItems="center" columnSpacing={2}>
      <Grid item xs={3}>
        <Typography>{title}</Typography>
      </Grid>

      <Grid item xs>
        <TextField
          inputRef={ref}
          type="color"
          variant="standard"
          fullWidth
          defaultValue={params[type]}
          InputProps={{ disableUnderline: true }}
        />
      </Grid>
    </Grid>
  );
}
