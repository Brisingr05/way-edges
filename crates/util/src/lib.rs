pub mod color;
pub mod draw;
pub mod template;
pub mod text;

pub mod shell {
    use log::{error, warn};
    use std::{os::unix::process::CommandExt, process::Command, thread};

    pub fn shell_cmd(value: &str) -> Result<String, String> {
        let mut cmd = Command::new("/bin/sh");
        log::debug!("running command: {value}");
        let res = cmd.arg("-c").arg(value).output();
        let msg = match res {
            Ok(o) => {
                if !o.status.success() {
                    Err(format!(
                        "command exit with code 1: {}",
                        String::from_utf8_lossy(&o.stderr)
                    ))
                } else {
                    Ok(String::from_utf8_lossy(&o.stdout).to_string())
                }
            }
            Err(e) => Err(format!("Error: {e}")),
        };
        if let Err(ref e) = msg {
            log::error!("error running command: {value}\n{e}");
        };
        msg
    }
    pub fn shell_cmd_non_block(value: String) {
        thread::spawn(move || {
            log::debug!("running command: {value}");
            let mut cmd = Command::new("/bin/sh");
            cmd.arg("-c").arg(&value);
            cmd.stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .stdin(std::process::Stdio::null());
            unsafe {
                cmd.pre_exec(move || {
                    match libc::fork() {
                        -1 => return Err(std::io::Error::last_os_error()),
                        0 => (),
                        _ => libc::_exit(0),
                    }

                    Ok(())
                });
            }
            match cmd.spawn() {
                Ok(mut c) => match c.wait() {
                    Ok(s) => {
                        if !s.success() {
                            warn!("command exit unsuccessfully: {value}");
                        }
                    }
                    Err(e) => {
                        error!("error waiting for command {cmd:?}: {e:?}");
                    }
                },
                Err(err) => {
                    error!("error spawning {cmd:?}: {err:?}");
                }
            }
        });
    }
}

pub static Z: f64 = 0.;

use std::fmt::Debug;
use std::fmt::Display;

pub fn binary_search_within_range<T: Debug + PartialOrd + Copy + Display>(
    l: &[[T; 2]],
    v: T,
) -> isize {
    if l.is_empty() {
        return -1;
    }
    if l.len() == 1 {
        if v >= l[0][0] && v < l[0][1] {
            return 0;
        } else {
            return -1;
        }
    }

    let mut index = l.len() - 1;
    let mut half = l.len();

    fn half_index(index: &mut usize, half: &mut usize, is_left: bool) {
        *half = (*half / 2).max(1);

        if is_left {
            *index -= *half
        } else {
            *index += *half
        }
    }

    half_index(&mut index, &mut half, true);

    loop {
        let current = l[index];

        if v < current[0] {
            if index == 0 || l[index - 1][1] <= v {
                return -1;
            } else {
                half_index(&mut index, &mut half, true);
            }
        } else if v >= current[1] {
            if index == l.len() - 1 || v < l[index + 1][0] {
                return -1;
            } else {
                half_index(&mut index, &mut half, false);
            }
        } else {
            return index as isize;
        }
    }
}

pub fn binary_search_end<T: Debug + PartialOrd + Copy + Display + Default>(l: &[T], v: T) -> isize {
    if l.is_empty() {
        return -1;
    }
    if l.len() == 1 {
        if v >= T::default() && v < l[0] {
            return 0;
        } else {
            return -1;
        }
    }

    let mut index = 0;
    let max_index = l.len() - 1;
    let mut get_half = {
        let mut half = l.len();
        move || {
            half = (half / 2).max(1);
            half
        }
    };

    loop {
        let current = l[index];

        if v < current {
            // if at the first, or there's no smaller to the left
            if index == 0 || v >= l[index - 1] {
                return index as isize;
            }
            index -= get_half();
        } else {
            // if it's the last
            if index == max_index {
                return -1;
            }

            // if smaller than the right
            if v < l[index + 1] {
                return (index + 1) as isize;
            }
            index += get_half();
        }
    }
}

#[derive(Debug)]
pub struct Or(pub bool);
impl Or {
    pub fn or(&mut self, b: bool) {
        self.0 = self.0 || b
    }
    pub fn res(self) -> bool {
        self.0
    }
}

/// input: rgba
/// output: bgra
pub fn pre_multiply_and_to_little_endian_argb(rgba: [u8; 4]) -> [u8; 4] {
    // pre-multiply
    let red = rgba[0] as u16;
    let green = rgba[1] as u16;
    let blue = rgba[2] as u16;
    let alpha = rgba[3] as u16;

    let r = (red * alpha) / 255;
    let g = (green * alpha) / 255;
    let b = (blue * alpha) / 255;

    // little-endian for ARgb32
    [b as u8, g as u8, r as u8, rgba[3]]
}
