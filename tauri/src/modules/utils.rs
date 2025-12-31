use objc2::rc::Retained;
use objc2_app_kit::NSColor;

pub fn format_speed(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{:>3} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:>3} K/s", bytes / 1024)
    } else {
        format!("{:>3.1} M/s", bytes as f64 / 1024.0 / 1024.0)
    }
}

pub fn format_size(bytes: usize) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{:.1} MB", bytes as f64 / 1024.0 / 1024.0)
    }
}

pub fn get_cpu_color(cpu: f32) -> Retained<NSColor> {
    if cpu >= 80.0 {
        NSColor::yellowColor()
    } else if cpu >= 50.0 {
        NSColor::orangeColor()
    } else {
        NSColor::controlTextColor()
    }
}

pub fn get_network_color(bytes_per_sec: u64) -> Retained<NSColor> {
    let mb_per_sec = bytes_per_sec as f64 / (1024.0 * 1024.0);
    if mb_per_sec > 10.0 {
        NSColor::redColor()
    } else if mb_per_sec >= 5.0 {
        NSColor::orangeColor()
    } else {
        NSColor::controlTextColor()
    }
}
