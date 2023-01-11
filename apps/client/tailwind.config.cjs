/**
 * @type {import('tailwindcss').Config}}}
 */
module.exports = {
  content: ["./src/**/*.{js,jsx,ts,tsx}"],
  theme: {
    extend: {
      backgroundImage: {
        arrow: "url('/images/svg/arrow.svg')",
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
      animation: {
        fadeIn: "fadeIn 150ms ease-in forwards",
        fadeInSlow: "fadeIn 1s ease-in forwards",
      },
      keyframes: {
        fadeIn: {
          "0%": { opacity: 0, transform: "translateY(10px)" },
          "100%": { opacity: 1, transform: "translateY(0)" },
        },
      },
    },
  },
  plugins: [],
};
