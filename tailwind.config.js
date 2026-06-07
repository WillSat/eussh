/** @type {import('tailwindcss').Config} */
export default {
  content: ['./index.html', './src/**/*.{vue,js,ts,jsx,tsx}'],
  darkMode: 'class',
  theme: {
    extend: {
      fontFamily: {
        sans: ['-apple-system', 'BlinkMacSystemFont', '"SF Pro Text"', '"SF Pro Display"', '"Helvetica Neue"', 'Helvetica', 'Arial', 'sans-serif'],
        mono: ['"SF Mono"', '"Fira Code"', '"Cascadia Code"', 'Consolas', 'monospace'],
      },
      colors: {
        accent: { DEFAULT: '#007AFF', hover: '#0066D6' },
      },
    },
  },
  plugins: [],
}
