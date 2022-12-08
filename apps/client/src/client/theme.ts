import { lightTheme, Theme } from "@rainbow-me/rainbowkit";
import { ThemeOptions } from "@rainbow-me/rainbowkit/dist/themes/baseTheme";

const options: ThemeOptions = {
  accentColor: "#7dd3fc",
  accentColorForeground: "#000000",
  fontStack: "system",
  borderRadius: "large",
  overlayBlur: "small",
};

export const theme: Theme = lightTheme(options);
