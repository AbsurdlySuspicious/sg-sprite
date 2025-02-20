use sg_sprite::*;

fn main() {
    if let Err(e) = lib_main(&Opts::from_args()) {
        print_err(e);
    }
}
