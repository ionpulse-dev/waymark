# Cask template for the menu bar app (ionpulse-dev/homebrew-tap, file Casks/waymark-bar.rb).
# scripts/release.sh --with-app replaces VERSION and SHA256 (see specs/features/menu-bar-app).
cask "waymark-bar" do
  version "VERSION"
  sha256 "SHA256"

  url "https://github.com/ionpulse-dev/waymark/releases/download/v#{version}/WaymarkBar-#{version}.zip"
  name "Waymark Bar"
  desc "Menu bar app that shows a waymark memory repository and your agents' writes"
  homepage "https://github.com/ionpulse-dev/waymark"

  depends_on macos: ">= :sonoma"
  depends_on arch: :arm64
  depends_on formula: "ionpulse-dev/tap/waymark"

  app "WaymarkBar.app"

  zap trash: "~/Library/Preferences/dev.waymark.bar.plist"
end
