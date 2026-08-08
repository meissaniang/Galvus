# Généré automatiquement à chaque release par .github/workflows/release.yml.
cask "galvus" do
  arch arm: "aarch64", intel: "x64"

  version "0.2.0"
  sha256 arm:   "d334d3fccf90e3688b977a11629cbcb3acb1b91847b3bfaf98666a3e8414914a",
         intel: "0eb96f584cc0ac57d522ae66af25655022509677946c4ae0b9e796b0efed424a"

  url "https://github.com/meissaniang/Galvus/releases/download/v#{version}/Galvus_#{version}_#{arch}-macos.dmg",
      verified: "github.com/meissaniang/Galvus/"
  name "Galvus"
  desc "Client SSH desktop moderne, open source et 100 % local"
  homepage "https://github.com/meissaniang/Galvus"

  # Permet à Homebrew de repérer la dernière release publiée sur GitHub.
  livecheck do
    url :url
    strategy :github_latest
  end

  app "Galvus.app"

  # L'application n'est pas signée par un certificat Apple : sans cela, macOS
  # afficherait « Galvus est endommagé » au premier lancement.
  postflight do
    system_command "/usr/bin/xattr",
                   args: ["-dr", "com.apple.quarantine", "#{appdir}/Galvus.app"]
  end

  zap trash: [
    "~/Library/Application Support/com.galvus.app",
    "~/Library/Caches/com.galvus.app",
    "~/Library/Logs/com.galvus.app",
    "~/Library/Preferences/com.galvus.app.plist",
    "~/Library/Saved Application State/com.galvus.app.savedState",
    "~/Library/WebKit/com.galvus.app",
  ]
end
