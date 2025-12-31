cask "pulse" do
  version "0.0.1"
  sha256 :no_check  # 首次发布后用实际 sha256 替换

  # Apple Silicon
  if Hardware::CPU.arm?
    url "https://github.com/ultrasev/pulse/releases/download/v#{version}/Pulse_#{version}_aarch64.dmg"
  else
    url "https://github.com/ultrasev/pulse/releases/download/v#{version}/Pulse_#{version}_x64.dmg"
  end

  name "Pulse"
  desc "Lightweight system monitoring desktop app"
  homepage "https://github.com/ultrasev/pulse"

  app "Pulse.app"

  zap trash: [
    "~/Library/Application Support/com.pulse.dev",
    "~/Library/Caches/com.pulse.dev",
    "~/Library/Preferences/com.pulse.dev.plist",
  ]
end
