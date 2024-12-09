import daisyui from "daisyui";

/** @type {import('tailwindcss').Config} */
export default {
  content: {
    relative: true,
    files: ["*.html", "./src/**/*.rs"],
  },
  darkMode: ["class"],
  plugins: [daisyui],
  daisyui: {
    themes: ["light"]
  }
};
