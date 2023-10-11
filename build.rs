fn main() {
    cc::Build::new()
        .file("ipf_utility.c")
        .compile("ipf_utility");
}
