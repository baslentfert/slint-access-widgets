//! Prints the open Win32 popup menu (window class `#32768`) as screen readers
//! like NVDA read it: through MSAA (`IAccessible`). Windows only; prints
//! nothing when no menu is open.
//!
//! States: 0x1 unavailable, 0x4 focused, 0x10 checked, 0x80 hot-tracked.

#[cfg(windows)]
fn main() -> windows::core::Result<()> {
    use windows::Win32::System::Com::{COINIT_APARTMENTTHREADED, CoInitializeEx};
    use windows::core::VARIANT;
    use windows::Win32::UI::Accessibility::{AccessibleObjectFromWindow, GetRoleTextW, IAccessible};
    use windows::Win32::UI::WindowsAndMessaging::{FindWindowW, OBJID_CLIENT};
    use windows::core::{Interface, w};

    unsafe {
        let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
        let Ok(menu) = FindWindowW(w!("#32768"), None) else { return Ok(()) };
        let mut raw = std::ptr::null_mut();
        AccessibleObjectFromWindow(menu, OBJID_CLIENT.0 as u32, &IAccessible::IID, &mut raw)?;
        let acc = IAccessible::from_raw(raw);
        let role = |child: i32| -> String {
            let v = acc.get_accRole(&VARIANT::from(child)).ok();
            let n = v.as_ref().and_then(|v| i32::try_from(v).ok()).unwrap_or(0) as u32;
            let mut buf = [0u16; 64];
            let len = GetRoleTextW(n, Some(&mut buf)) as usize;
            String::from_utf16_lossy(&buf[..len])
        };
        let name = |child: i32| acc.get_accName(&VARIANT::from(child)).map(|b| b.to_string()).unwrap_or_default();
        let state = |child: i32| acc.get_accState(&VARIANT::from(child)).ok().and_then(|v| i32::try_from(&v).ok()).unwrap_or(0);
        let focus = acc.accFocus().ok().and_then(|v| i32::try_from(&v).ok()).unwrap_or(0);
        println!("  native menu: {} '{}', focus on item {focus}", role(0), name(0));
        for i in 1..=acc.accChildCount()? {
            println!("    {} '{}' state=0x{:x}", role(i), name(i), state(i));
        }
    }
    Ok(())
}

#[cfg(not(windows))]
fn main() {
    eprintln!("msaa-menu is for Windows");
}
