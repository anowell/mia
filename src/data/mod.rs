pub use self::cat::Cat;
pub use self::cp::Cp;
pub use self::ls::Ls;
pub use self::mkdir::MkDir;
pub use self::rm::Rm;
pub use self::rmdir::RmDir;

mod cat;
mod cp;
mod ls;
mod mkdir;
mod rm;
mod rmdir;

pub fn size_with_suffix(size: u64) -> String {
    match size / 1024 {
        0 => format!("{}", size),
        k if k < 10 => format!("{:.1}k", (size as f64) / 1024.0),
        k => match k / 1024 {
            0 => format!("{}k", k),
            m if m < 10 => format!("{:.1}M", (k as f64) / 1024.0),
            m => match m / 1024 {
                0 => format!("{}M", m),
                g if g < 10 => format!("{:.1}G", (m as f32) / 1024.0),
                g => format!("{}G", g),
            },
        },
    }
}
