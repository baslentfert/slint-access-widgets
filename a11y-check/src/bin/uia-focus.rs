//! Prints the element with the keyboard focus as UI Automation gives it to
//! screen readers (the COM interface, as NVDA uses it): type, name, help
//! text and full description (which the .NET client doesn't know).
//! Windows only.

#[cfg(windows)]
fn main() -> windows::core::Result<()> {
    use windows::Win32::System::Com::{CLSCTX_INPROC_SERVER, COINIT_APARTMENTTHREADED, CoCreateInstance, CoInitializeEx};
    use windows::Win32::System::Variant::VT_BSTR;
    use windows::Win32::UI::Accessibility::{CUIAutomation, IUIAutomation, UIA_FullDescriptionPropertyId};

    unsafe {
        let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
        let uia: IUIAutomation = CoCreateInstance(&CUIAutomation, None, CLSCTX_INPROC_SERVER)?;
        let e = uia.GetFocusedElement()?;
        let kind = e.CurrentLocalizedControlType().map(|b| b.to_string()).unwrap_or_default();
        let name = e.CurrentName().map(|b| b.to_string()).unwrap_or_default();
        let help = e.CurrentHelpText().map(|b| b.to_string()).unwrap_or_default();
        let v = e.GetCurrentPropertyValue(UIA_FullDescriptionPropertyId)?;
        let full = if v.Anonymous.Anonymous.vt == VT_BSTR {
            v.Anonymous.Anonymous.Anonymous.bstrVal.to_string()
        } else {
            String::new()
        };
        println!("  uia: {kind} '{name}' help='{help}' full-description='{full}'");
    }
    Ok(())
}

#[cfg(not(windows))]
fn main() {
    eprintln!("uia-focus is for Windows");
}
