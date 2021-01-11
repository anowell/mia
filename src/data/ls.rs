use crate::config::Profile;
use crate::{color_choice, data, CmdRunner};
use algorithmia::data::{DataItem, HasDataPath};
use algorithmia::Algorithmia;
use docopt::Docopt;
use std::cmp;
use std::io::Write;
use std::ops::Deref;
use std::vec::IntoIter;
use termcolor::{Color, ColorSpec, StandardStream, WriteColor};
use terminal_size::{terminal_size, Width};

static USAGE: &'static str = r##"Usage:
  algo ls [options] [<data-dir>]
  algo dir [options] [<data-dir>]

  List contents of a directory via the Agorithmia Data API

  <data-dir>    Specifies the Algorithmia Data URI
                The 'data://' prefix is optional
                Defaults to 'data://' root path

  Options:
    -l          Use long listing format
"##;

#[derive(RustcDecodable, Debug)]
struct Args {
    arg_data_dir: Option<String>,
    flag_l: bool,
}

pub struct Ls {
    client: Algorithmia,
}

impl CmdRunner for Ls {
    fn get_usage() -> &'static str {
        USAGE
    }

    fn cmd_main(&self, argv: IntoIter<String>) {
        let args: Args = Docopt::new(USAGE)
            .and_then(|d| d.argv(argv).decode())
            .unwrap_or_else(|e| e.exit());

        let data_uri = args
            .arg_data_dir
            .as_ref()
            .map(Deref::deref)
            .unwrap_or("data://");
        self.list_dir(data_uri, args.flag_l);
    }
}

impl Ls {
    pub fn new(profile: Profile) -> Self {
        Ls {
            client: profile.client(),
        }
    }

    fn list_dir(&self, path: &str, long: bool) {
        let my_dir = self.client.dir(path);

        let mut t_out = StandardStream::stdout(color_choice());
        if long {
            for entry_result in my_dir.list() {
                match entry_result {
                    Ok(DataItem::Dir(d)) => {
                        let _ = write!(t_out, "{:19} {:>5} ", "--         --", "[dir]");
                        let _ = t_out.set_color(ColorSpec::new().set_fg(Some(Color::Blue)));
                        let _ = writeln!(t_out, "{}", d.basename().unwrap());
                        let _ = t_out.reset();
                    }
                    Ok(DataItem::File(f)) => {
                        let name = f.basename().unwrap();
                        let _ = write!(
                            t_out,
                            "{:19} {:>5} ",
                            f.last_modified.format("%Y-%m-%d %H:%M:%S"),
                            data::size_with_suffix(f.size)
                        );
                        let c = FileType::from_filename(&name).to_color();
                        let _ = t_out.set_color(ColorSpec::new().set_fg(c));
                        let _ = writeln!(t_out, "{}", name);
                        let _ = t_out.reset();
                    }
                    Err(err) => quit_err!("Error listing directory: {}", err),
                }
            }
        } else {
            let items: Vec<DataItem> = my_dir
                .list()
                .collect::<Result<Vec<_>, _>>()
                .unwrap_or_else(|err| quit_err!("Error listing directory: {}", err));

            let width = match terminal_size() {
                Some((Width(w), _)) => w as usize,
                _ => 80, // default terminal width if we can't calculate it
            };

            let max_len = items.iter().fold(0, |max, item| {
                let name_len = match *item {
                    DataItem::File(ref f) => f.basename().unwrap().len(),
                    DataItem::Dir(ref d) => d.basename().unwrap().len(),
                };
                cmp::max(max, name_len)
            });
            let col_width = max_len + 2;

            let mut offset = 0;
            for item in items {
                if offset + col_width > width {
                    let _ = writeln!(t_out, "");
                    offset = 0;
                }
                let char_count = match item {
                    DataItem::Dir(d) => {
                        let name = d.basename().unwrap();
                        let _ = t_out.set_color(ColorSpec::new().set_fg(Some(Color::Blue)));
                        let _ = write!(t_out, "{}", name);
                        let _ = t_out.reset();
                        name.chars().count()
                    }
                    DataItem::File(f) => {
                        let name = f.basename().unwrap();
                        let c = FileType::from_filename(&name).to_color();
                        let _ = t_out.set_color(ColorSpec::new().set_fg(c));
                        let _ = write!(t_out, "{}", name);
                        let _ = t_out.reset();
                        name.chars().count()
                    }
                };
                if char_count < col_width {
                    let _ = write!(t_out, "{:1$}", "", col_width - char_count);
                }

                offset += col_width;
            }

            let _ = writeln!(t_out, "");
        }
    }
}

enum FileType {
    Image,
    Video,
    Archive,
    Audio,
    Unknown,
}

impl FileType {
    fn from_filename(filename: &str) -> FileType {
        filename
            .rsplit('.')
            .next()
            .map(FileType::from_ext)
            .unwrap_or(FileType::Unknown)
    }

    // A basic mime guessing function
    fn from_ext(ext: &str) -> FileType {
        match &*ext.to_lowercase() {
            "bmp" | "gif" | "ico" | "jpe" | "jpeg" | "jpg" | "png" | "svg" | "tif" | "tiff"
            | "webp" | "xcf" | "psd" | "ai" => FileType::Image,

            "3g2" | "3gp" | "avi" | "divx" | "flv" | "mov" | "mp4" | "mp4v" | "mpa" | "mpe"
            | "mpeg" | "ogv" | "qt" | "webm" | "wmv" => FileType::Video,

            "7z" | "rar" | "tgz" | "gz" | "zip" | "tar" | "xz" | "dmg" | "iso" | "lzma" | "tlz"
            | "bz2" | "tbz2" | "z" | "deb" | "rpm" | "jar" => FileType::Archive,

            "aac" | "flac" | "ogg" | "au" | "mid" | "midi" | "mp3" | "mpc" | "ra" | "wav"
            | "axa" | "oga" | "spz" | "xspf" | "wma" | "m4a" => FileType::Audio,

            _ => FileType::Unknown,
        }
    }

    fn to_color(&self) -> Option<Color> {
        match *self {
            FileType::Image | FileType::Video => Some(Color::Magenta),
            FileType::Archive => Some(Color::Red),
            FileType::Audio => Some(Color::Cyan),
            _ => None,
        }
    }
}
