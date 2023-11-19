use crate::ies_parser::IesFile;

mod ies_parser;

fn main() {
    let ies_data =
        IesFile::load_from_file("C:\\Users\\Ridwan Hidayatullah\\Videos\\tosmole\\tests\\cell.ies")
            .unwrap();
    println!("{:?}", &ies_data);
    println!("columns len : {}", &ies_data.get_columns_length().unwrap());
    println!("rows len : {}", &ies_data.get_rows_length().unwrap());
    if let Some(data) = &ies_data.get_data_by_column_name_and_index("Script", 3) {
        println!("Data: {:?}", data);
    } else {
        println!("Column or row not found");
    }
    let column_names = &ies_data.get_column_names();
    for name in column_names {
        println!("Column Name: {}", name);
    }
}
