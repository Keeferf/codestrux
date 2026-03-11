import type { CreativityModes } from "../types";

export const CREATIVITY_MODES: CreativityModes = {
  precise: {
    label: "Precise",
    temp: 0.2,
    desc: "deterministic · factual",
    accent: "var(--color-indigo-smoke-500)",
  },
  balanced: {
    label: "Balanced",
    temp: 0.7,
    desc: "default · general use",
    accent: "var(--color-khaki-beige-500)",
  },
  creative: {
    label: "Creative",
    temp: 1.2,
    desc: "expressive · exploratory",
    accent: "var(--color-amber-dust-500)",
  },
};
