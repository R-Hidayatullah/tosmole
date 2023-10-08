fn main() {
    cc::Build::new()
        .file("ipf_decrypt.c")
        .compile("ipf_decrypt");
}
