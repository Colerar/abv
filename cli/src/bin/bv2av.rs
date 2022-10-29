use abv::bv2av;
use clap::Parser;

#[derive(Parser, Debug, Clone)]
#[clap(version, name = "bv2av")]
struct Cli {
  #[arg(value_name = "BVID", num_args = 1..)]
  bvids: Vec<String>,
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
  dbg!(&args);

  for bv in args.bvids.into_iter() {
    match bv2av(&*bv) {
      Ok(av) => {
        let prefix = if !args.no_prefix { "av" } else { "" };
        let suffix = if !args.no_suffix {
          format!(" = {}", bv)
        } else {
          "".to_string()
        };
        print!("{}{}{}", prefix, av, suffix)
      },
      Err(err) => eprint!("Failed to convert bv: {}, {}", bv, err),
    }
    print!("{}", args.separator);
  }
}
