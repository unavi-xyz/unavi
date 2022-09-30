/**
 * @type {import('tailwindcss').Config}}}
 */
module.exports = {
  content: ["./src/**/*.{js,jsx,ts,tsx}"],
  theme: {
    extend: {
      colors: {
        primary: "#52DAFF",
        primaryContainer: "#C5E8F5",
        onPrimary: "#000000",
        onPrimaryContainer: "#000000",

        surface: "#ffffff",
        surfaceVariant: "#ebeced",
        onSurface: "#000000",
        onSurfaceVariant: "#000000",

        surfaceDark: "#000000",
        onSurfaceDark: "#ffffff",

        background: "#ffffff",
        onBackground: "#000000",

        error: "#B3261E",
        errorContainer: "#F9DEDC",
        onError: "#ffffff",
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
        dark: "0 1px 3px 0 rgb(0 0 0 / 0.3)",
      },
      dropShadow: {
        dark: "0 2px 16px rgba(0, 0, 0, 0.8)",
      },
    },
  },
  plugins: [],
};
