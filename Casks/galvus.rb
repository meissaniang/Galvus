# Généré automatiquement à chaque release par .github/workflows/release.yml.
cask "galvus" do
  arch arm: "aarch64", intel: "x64"

  version "0.1.0"
  sha256 arm:   "6e36fad390ab3b3e3ca4876e3afbe7d8579ccf5a7cadf7d094b95bc4a0be5462",
         intel: "a4916568beaacbc657425585d018c0e772ca8eeb390f837f0b59ef90e3f61559"

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
