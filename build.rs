extern crate cc;

fn main() {
    cc::Build::new()
        .file("src/unix/extern_seccomp.c")
        .compile("seccomp");

    cc::Build::new()
        .file("src/unix/extern_open.c")
        .compile("open");
}
