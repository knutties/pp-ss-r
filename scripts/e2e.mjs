// Minimal, dependency-free e2e intent doc.
// The canonical check is the shell command in README ("Test" section):
//   "$CHROME_BIN" --headless --disable-gpu --screenshot=scratch/home.png \
//       --window-size=480,900 http://127.0.0.1:8080
// On Linux the Nix devShell provides chromium; on darwin CHROME_BIN points at
// the system Google Chrome. This file is a placeholder for a future CDP-driven
// assertion script.
console.log("Run the headless screenshot command from README.md (needs a running server).");
