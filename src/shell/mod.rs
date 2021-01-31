use self::shell::Shell;

pub mod bash;
mod detect;
pub mod powershell;
pub mod shell;
pub mod windows_cmd;
pub mod zsh;

#[cfg(windows)]
pub fn detect_shell() -> Option<Box<dyn Shell>> {
    self::detect::windows::detect_shell()
}

#[cfg(unix)]
pub fn detect_shell() -> Option<Box<dyn Shell>> {
    self::detect::unix::detect_shell()
}
