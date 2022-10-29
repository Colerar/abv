use abv::av2bv;
use clap::Parser;

#[derive(Parser, Debug, Clone)]
#[clap(version, name = "bv2av")]
struct Cli {
  #[arg(value_name = "AVID", num_args = 1..)]
  avids: Vec<String>,
  /// Do not show `av` prefix
  #[arg(short = 'P', long = "no-prefix", default_value_t = false)]
  no_prefix: bool,
  /// Do not show ` = BVxxx` prefix
  #[arg(short = 'S', long = "no-suffix", default_value_t = false)]
  no_suffix: bool,
  /// Separator of each element
  #[arg(
    short = 's',
    long = "separator",
    value_name = "SEP",
    default_value = "\n"
  )]
  separator: String,
}

fn main() {
  let args: Cli = Cli::parse();
  #[cfg(debug_assertions)]
  dbg!(&args);

  for av in args.avids.into_iter() {
    let av = match av.trim().trim_start_matches("av").parse::<u64>() {
      Ok(av) => av,
      Err(err) => {
        eprint!("Failed to parse arg as int64: {}", err);
        eprint!("{}", &args.separator);
        continue;
      },
    };
    match av2bv(av) {
      Ok(bv) => {
        let prefix = if !args.no_prefix {
          "av"
        } else {
          ""
        };
        let suffix = if !args.no_suffix {
          format!(" = {}{}", prefix, av)
        } else {
          "".to_string()
        };
        print!("{}{}", bv, suffix)
      },
      Err(err) => eprint!("Failed to convert av: {}, {}", av, err),
    }
    print!("{}", args.separator);
  }
}
