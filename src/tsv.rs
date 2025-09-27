use std::{
    fs::File,
    io::{self, BufRead, BufReader},
    path::Path,
    thread,
};

pub fn parse_tsv_file(path: &Path) -> std::io::Result<Vec<Vec<String>>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    let mut rows = Vec::new();
    for line in reader.lines() {
        let line = line?;
        // Split on tabs and collect columns as Strings
        let cols: Vec<String> = line.split('\t').map(|s| s.to_string()).collect();
        rows.push(cols);
    }
    Ok(rows)
}

pub fn parse_language_data_parallel(
    folder: &Path,
) -> std::io::Result<(Vec<Vec<String>>, Vec<Vec<String>>)> {
    let etc_path = folder.join("ETC.tsv");
    let item_path = folder.join("ITEM.tsv");

    let handle_etc = thread::spawn(move || parse_tsv_file(&etc_path));
    let handle_item = thread::spawn(move || parse_tsv_file(&item_path));

    let etc_data = handle_etc.join().expect("ETC thread panicked")?;
    let item_data = handle_item.join().expect("ITEM thread panicked")?;

    Ok((etc_data, item_data))
}

pub fn parse_language_data(lang_folder: &Path) -> io::Result<(Vec<Vec<String>>, Vec<Vec<String>>)> {
    parse_language_data_parallel(lang_folder)
}
