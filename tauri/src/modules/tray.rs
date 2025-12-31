use tauri::{AppHandle, Manager};
use objc2::{rc::Allocated, MainThreadMarker, ClassType};
use objc2_foundation::{
    ns_string, NSDictionary, NSMutableAttributedString, NSString, NSRange,
};
use objc2::runtime::AnyObject;
use crate::modules::AppState;

// Re-export utilities for use in other modules
pub use crate::modules::utils::{format_speed, get_cpu_color, get_network_color};

pub fn update_status_bar(app: &AppHandle, cpu: f32, up: u64, down: u64) {
    let cpu_str = format!("{:.0}%", cpu);
    let up_str = format!("{}", format_speed(up));
    let down_str = format!("{}", format_speed(down));

    let sep1 = ",";
    let sep2 = ",";

    let cpu_len = cpu_str.encode_utf16().count();
    let sep1_len = sep1.encode_utf16().count();
    let up_len = up_str.encode_utf16().count();
    let sep2_len = sep2.encode_utf16().count();
    let down_len = down_str.encode_utf16().count();

    let full_text = format!("{}{}{}{}{}", cpu_str, sep1, up_str, sep2, down_str);

    let handle = app.clone();

    let _ = app.run_on_main_thread(move || {
        let mtm = unsafe { MainThreadMarker::new_unchecked() };

        let state = handle.state::<AppState>();
        let lock = state.status_item.lock().unwrap();

        if let Some(wrapper) = lock.as_ref() {
            let item = &wrapper.0;

            let full_ns = NSString::from_str(&full_text);

            let alloc_mut: Allocated<NSMutableAttributedString> = unsafe {
                objc2::msg_send![NSMutableAttributedString::class(), alloc]
            };
            let mut_attr_str = NSMutableAttributedString::initWithString(alloc_mut, &full_ns);

            // Apply CPU color
            let cpu_range = NSRange::new(0, cpu_len);
            let cpu_key = ns_string!("NSColor");
            let cpu_dict = NSDictionary::from_slices(&[cpu_key], &[&*get_cpu_color(cpu)]);
            let cpu_dict_ptr: &NSDictionary<NSString, AnyObject> = unsafe { std::mem::transmute(&*cpu_dict) };
            unsafe {
                mut_attr_str.setAttributes_range(Some(cpu_dict_ptr), cpu_range);
            }

            // Apply upload color
            let up_start = cpu_len + sep1_len;
            let up_range = NSRange::new(up_start, up_len);
            let up_dict = NSDictionary::from_slices(&[cpu_key], &[&*get_network_color(up)]);
            let up_dict_ptr: &NSDictionary<NSString, AnyObject> = unsafe { std::mem::transmute(&*up_dict) };
            unsafe {
                mut_attr_str.setAttributes_range(Some(up_dict_ptr), up_range);
            }

            // Apply download color
            let down_start = up_start + up_len + sep2_len;
            let down_range = NSRange::new(down_start, down_len);
            let down_dict = NSDictionary::from_slices(&[cpu_key], &[&*get_network_color(down)]);
            let down_dict_ptr: &NSDictionary<NSString, AnyObject> = unsafe { std::mem::transmute(&*down_dict) };
            unsafe {
                mut_attr_str.setAttributes_range(Some(down_dict_ptr), down_range);
            }

            if let Some(button) = item.button(mtm) {
                button.setAttributedTitle(&mut_attr_str);
            }
        }
    });
}
