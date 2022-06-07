module.exports = {
  mode: "jit",
  content: ["./pages/**/*.{ts,tsx}", "./src/**/*.{ts,tsx}"],
  theme: {
    extend: {
      colors: {
        primary: "#6750A4",
        primaryContainer: "#EADDFF",
        onPrimary: "#FFFFFF",
        onPrimaryContainer: "#21005E",

        secondary: "#625B71",
        secondaryContainer: "#E8DEF8",
        onSecondary: "#FFFFFF",
        onSecondaryContainer: "#1E192B",

        tertiary: "#7D5260",
        tertiaryContainer: "#FFD8E4",
        onTertiary: "#FFFFFF",
        onTertiaryContainer: "#370B1E",

        surface: "#FFFBFE",
        surfaceVariant: "#E7E0EC",
        onSurface: "#1C1B1F",
        onSurfaceVariant: "#49454E",

        surfaceDark: "#1C1B1F",
        onSurfaceDark: "#E6E1E5",

        background: "#FFFBFE",
        onBackground: "#1C1B1F",

        error: "#B3261E",
        errorContainer: "#F9DEDC",
        onError: "#FFFFFF",
        onErrorContainer: "#370B1E",

        outline: "#79747E",
      },
      backgroundImage: {
        arrow: "url('/images/arrow.svg')",
      },
      aspectRatio: {
        card: "5/3",
        vertical: "3/5",
      },
      boxShadow: {
        tonal: "0 1px 3px 0 rgb(0 0 0 / 0.3)",
        filled: "0 1px 3px 0 rgb(0 0 0 / 0.5)",
      },
    },
  },
  plugins: [],
};
