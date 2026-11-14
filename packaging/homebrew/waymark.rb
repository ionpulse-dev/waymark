# Formula template for the waymark tap (ionpulse-dev/homebrew-tap, file Formula/waymark.rb).
# The release workflow and scripts/brew-local.sh replace VERSION and SHA256 (and URL for local tests).
class Waymark < Formula
  desc "Knowledge base and memory for AI agents: an MCP server over Markdown in Git"
  homepage "https://github.com/ionpulse-dev/waymark"
  url "https://github.com/ionpulse-dev/waymark/releases/download/vVERSION/waymark-VERSION-aarch64-apple-darwin.tar.gz"
  sha256 "SHA256"
  license "MIT"

  depends_on arch: :arm64
  depends_on :macos

  def install
    bin.install "waymark"
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/waymark --version")
  end
end
