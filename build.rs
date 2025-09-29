fn main() {
    cc::Build::new()
        .file("c_src/stb_image_wrapper.c")
        .include("c_src")
        .compile("stb_image_wrapper");
}
