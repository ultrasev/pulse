cask "pulse" do
  version "0.0.3"
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

  postflight do
    system_command "/usr/bin/xattr",
                   args: ["-d", "com.apple.quarantine", "#{appdir}/Pulse.app"],
                   sudo: false
  end

  zap trash: [
    "~/Library/Application Support/com.pulse.dev",
    "~/Library/Caches/com.pulse.dev",
    "~/Library/Preferences/com.pulse.dev.plist",
  ]
end
