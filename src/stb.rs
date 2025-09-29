use std::os::raw::{c_int, c_uchar};

unsafe extern "C" {
    fn stb_load_from_memory(
        buffer: *const c_uchar,
        len: c_int,
        x: *mut c_int,
        y: *mut c_int,
        channels: *mut c_int,
    ) -> *mut c_uchar;

    fn stb_free_image(data: *mut c_uchar);

    fn stb_write_png_mem(
        pixels: *const c_uchar,
        w: c_int,
        h: c_int,
        comp: c_int,
        out_len: *mut c_int,
    ) -> *mut c_uchar;
}

pub struct Image {
    pub width: i32,
    pub height: i32,
    pub channels: i32,
    pub data: Vec<u8>,
}

pub fn load_tga_from_memory(bytes: &[u8]) -> Option<Image> {
    let mut x = 0;
    let mut y = 0;
    let mut channels = 0;

    unsafe {
        let ptr = stb_load_from_memory(
            bytes.as_ptr(),
            bytes.len() as i32,
            &mut x,
            &mut y,
            &mut channels,
        );
        if ptr.is_null() {
            return None;
        }

        let size = (x * y * channels) as usize;
        let slice = std::slice::from_raw_parts(ptr, size);
        let data = slice.to_vec();
        stb_free_image(ptr);

        Some(Image {
            width: x,
            height: y,
            channels,
            data,
        })
    }
}

pub fn encode_png_to_memory(img: &Image) -> Option<Vec<u8>> {
    let mut out_len = 0;
    unsafe {
        let ptr = stb_write_png_mem(
            img.data.as_ptr(),
            img.width,
            img.height,
            img.channels,
            &mut out_len,
        );
        if ptr.is_null() {
            return None;
        }

        let slice = std::slice::from_raw_parts(ptr, out_len as usize);
        let png = slice.to_vec();

        libc::free(ptr as *mut _); // free malloc'd buffer
        Some(png)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_tga_from_memory() {
        // Load a sample TGA file into memory
        let tga_bytes = std::fs::read("tests/enclass.tga").expect("missing tests/enclass.tga file");

        let img = load_tga_from_memory(&tga_bytes).expect("failed to decode TGA");

        assert!(img.width > 0);
        assert!(img.height > 0);
        assert!(img.channels >= 3 && img.channels <= 4);

        println!(
            "Decoded {}x{} ({} channels)",
            img.width, img.height, img.channels
        );
    }

    #[test]
    fn test_tga_to_png_conversion() {
        // Load TGA into memory
        let tga_bytes = std::fs::read("tests/enclass.tga").expect("missing tests/enclass.tga file");

        let img = load_tga_from_memory(&tga_bytes).expect("failed to decode TGA");

        // Encode PNG to memory
        let png_bytes = encode_png_to_memory(&img).expect("failed to encode PNG");

        // PNG signature check (first 8 bytes)
        assert!(png_bytes.starts_with(&[137, 80, 78, 71, 13, 10, 26, 10]));

        println!("Converted TGA to PNG ({} bytes)", png_bytes.len());
    }
}
