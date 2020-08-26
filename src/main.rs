use sg_sprite::*;
use structopt::StructOpt;

fn main() {
    if let Err(e) = lib_main(&Opts::from_args()) {
        print_err(e);
    }
}
