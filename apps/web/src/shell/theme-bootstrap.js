(() => {
  var preference = document.documentElement.dataset.themePreference;
  var mode =
    preference === "DARK"
      ? "dark"
      : preference === "LIGHT"
        ? "light"
        : window.matchMedia("(prefers-color-scheme: dark)").matches
          ? "dark"
          : "light";
  document.documentElement.dataset.colorMode = mode;
})();
