import { defineStore } from "pinia";
import { ref, onMounted } from "vue";

export const accentColors = {
  blue: {
    label: "Cyber Blue",
    pulse: "rgb(79, 124, 255)",
    pulseDim: "rgb(42, 74, 204)",
    pulseGlow: "rgb(107, 147, 255)",
  },
  pink: {
    label: "Neon Pink",
    pulse: "rgb(255, 46, 204)",
    pulseDim: "rgb(204, 0, 153)",
    pulseGlow: "rgb(255, 107, 226)",
  },
  emerald: {
    label: "Emerald",
    pulse: "rgb(16, 185, 129)",
    pulseDim: "rgb(5, 150, 105)",
    pulseGlow: "rgb(52, 211, 153)",
  },
  amber: {
    label: "Amber",
    pulse: "rgb(245, 158, 11)",
    pulseDim: "rgb(217, 119, 6)",
    pulseGlow: "rgb(251, 191, 36)",
  },
} as const;

export type AccentColorKey = keyof typeof accentColors;

export const useSettingsStore = defineStore("settings", () => {
  const accentColor = ref<AccentColorKey>((localStorage.getItem("accent_color") as AccentColorKey) || "blue");
  const notificationsEnabled = ref(localStorage.getItem("notifications_enabled") === "true");
  const soundsEnabled = ref(localStorage.getItem("sounds_enabled") !== "false"); // Default to true

  function setAccentColor(color: AccentColorKey) {
    accentColor.value = color;
    localStorage.setItem("accent_color", color);
    applyTheme(color);
  }

  function setNotifications(enabled: boolean) {
    notificationsEnabled.value = enabled;
    localStorage.setItem("notifications_enabled", enabled.toString());
  }

  function setSounds(enabled: boolean) {
    soundsEnabled.value = enabled;
    localStorage.setItem("sounds_enabled", enabled.toString());
  }

  function applyTheme(colorKey: AccentColorKey) {
    const color = accentColors[colorKey];
    const root = document.documentElement;
    root.style.setProperty("--accent-color", color.pulse);
    root.style.setProperty("--accent-color-dim", color.pulseDim);
    root.style.setProperty("--accent-color-glow", color.pulseGlow);
    // Update shadow properties as they depend on the pulse color
    const pulseRgb = color.pulse.match(/\d+, \d+, \d+/)?.[0] || "79, 124, 255";
    root.style.setProperty("--shadow-glow", `0 0 20px rgb(${pulseRgb} / 0.15)`);
    root.style.setProperty("--shadow-glow-strong", `0 0 40px rgb(${pulseRgb} / 0.25)`);
  }

  // Initial application on mount
  onMounted(() => {
    applyTheme(accentColor.value);
  });

  return {
    accentColor,
    notificationsEnabled,
    soundsEnabled,
    setAccentColor,
    setNotifications,
    setSounds,
    applyTheme,
  };
});
