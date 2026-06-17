use std::path::PathBuf;
use stef_core::File;
use stef_core::value::bitsize::BitSize;
use stef_core::value::bool::Bool;
use stef_core::value::bytes::Bytes;
use stef_core::value::heteroarray::HeteroArray;
use stef_core::value::homoarray::HomoArray;
use stef_core::value::number::Number;
use stef_core::value::record::Record;
use stef_core::value::StefValue;
use stef_core::value::string::Text;

fn base_path() -> PathBuf {
    let mut path = std::env::current_dir().unwrap();
    path.push("tests/files/golden/");
    path
}

mod file_names {
    pub const SIMPLE_GOLDEN: &str = "simple.stef";
    pub const CHECKSUM_GOLDEN: &str = "checksum.stef";
    pub const COMPRESSED_GOLDEN: &str = "compressed.stef";
    pub const CHECKSUM_COMPRESSED_GOLDEN: &str = "checksum-compressed.stef";
}

fn golden_uint() -> Number {
    Number::uint(1234567890, stef_core::value::bitsize::BitSize::Double)
}

fn golden_int() -> Number {
    Number::int(1234567890, stef_core::value::bitsize::BitSize::Double)
}

fn golden_float32() -> Number {
    Number::float_single(f32::EPSILON)
}

fn golden_float64() -> Number {
    Number::float_double(std::f64::consts::PI)
}

fn golden_bool() -> Bool {
    Bool::new(true)
}

fn golden_string() -> Text {
    Text::new("Hello, world!".to_string())
}

fn golden_bytes() -> Bytes {
    Bytes::new(vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10])
}

fn golden_homo_array() -> HomoArray {
    let pi = Number::float_double_nullable(std::f64::consts::PI, true);
    let type_byte = pi.type_byte();
    HomoArray::new(vec![
        pi.into(),
        Number::float_double_nullable(std::f64::consts::E, true).into(),
    ], type_byte)
}

fn golden_hetero_array() -> HeteroArray {
    HeteroArray::new(vec![
        Number::float_double_nullable(std::f64::consts::PI, true).into(),
        Number::float_single(f32::EPSILON).into()
    ])
}

fn golden_record() -> Record {
    let mut record = Record::new_empty();
    let fields = record.fields_mut().unwrap();

    fields.insert("name".into(), Text::new("Johan".to_string()).into());
    fields.insert("email".into(), Text::new("johan@example.com").into());
    fields.insert("age".into(), Number::uint(18, BitSize::Mini).into());
    fields.insert("verified".into(), Bool::new(true).into());

    record
}

fn golden_file() -> File {
    let mut file = File::new_empty();
    let root = file.get_root_mut();

    root.insert("uint".into(), golden_uint().into());
    root.insert("nullable uint".into(), golden_uint().nullable().unwrap().into());
    root.insert("int".into(), golden_int().into());
    root.insert("nullable int".into(), golden_int().nullable().unwrap().into());
    root.insert("f32".into(), golden_float32().into());
    root.insert("nullable f32".into(), golden_float32().nullable().unwrap().into());
    root.insert("f64".into(), golden_float64().into());
    root.insert("nullable f64".into(), golden_float64().nullable().unwrap().into());
    root.insert("bool".into(), golden_bool().into());
    root.insert("string".into(), golden_string().into());
    root.insert("nullable string".into(), golden_string().nullable().unwrap().into());
    root.insert("bytes".into(), golden_bytes().into());
    root.insert("nullable bytes".into(), golden_bytes().nullable().unwrap().into());
    root.insert("homo array".into(), golden_homo_array().into());
    root.insert("hetero array".into(), golden_hetero_array().into());
    root.insert("record".into(), golden_record().into());
    root.insert("nullable record".into(), golden_record().nullable().unwrap().into());

    file
}

fn exists_golden_file(file: &str) -> bool {
    std::fs::metadata(base_path().join(file)).is_ok()
}

fn write_golden_file(name: &str, file: &File) {
    let serialized = file.serialize().unwrap();
    std::fs::write(base_path().join(name), &serialized).unwrap();
}

fn deserialize_golden_file(name: &str) -> File {
    let serialized = std::fs::read(base_path().join(name)).unwrap();
    File::deserialize(&mut &serialized[..]).unwrap()
}

fn read_and_compare_golden_file(name: &str, file: &File) {
    let serialized = std::fs::read(base_path().join(name)).unwrap();
    let file_serialized = file.serialize().unwrap();
    println!("File {}:", name);
    println!("Golden File size:  {}B", serialized.len());
    println!("Created File size: {}B", file_serialized.len());
    println!("Golden File content:  {:?}", serialized);
    println!("Created File content: {:?}", file_serialized);
    assert_eq!(serialized, file_serialized, "Files don't match for file: {}", name);
}

#[test]
fn read_and_compare_simple_golden_file() {
    if !exists_golden_file(file_names::SIMPLE_GOLDEN) {
        write_golden_file(file_names::SIMPLE_GOLDEN, &golden_file());
    }
    read_and_compare_golden_file(file_names::SIMPLE_GOLDEN, &golden_file());
}

#[test]
fn read_and_compare_compressed_golden_file() {
    if !exists_golden_file(file_names::COMPRESSED_GOLDEN) {
        let mut file = golden_file();
        file.flags.compressed = true;
        write_golden_file(file_names::COMPRESSED_GOLDEN, &file);
    }
    let mut file = golden_file();
    file.flags.compressed = true;
    read_and_compare_golden_file(file_names::COMPRESSED_GOLDEN, &file);
}

#[test]
fn read_and_compare_checksum_golden_file() {
    if !exists_golden_file(file_names::CHECKSUM_GOLDEN) {
        let mut file = golden_file();
        file.flags.checksum = true;
        write_golden_file(file_names::CHECKSUM_GOLDEN, &file);
    }
    let mut file = golden_file();
    file.flags.checksum = true;
    read_and_compare_golden_file(file_names::CHECKSUM_GOLDEN, &file);
}

#[test]
fn read_and_compare_checksum_compressed_golden_file() {
    if !exists_golden_file(file_names::CHECKSUM_COMPRESSED_GOLDEN) {
        let mut file = golden_file();
        file.flags.checksum = true;
        file.flags.compressed = true;
        write_golden_file(file_names::CHECKSUM_COMPRESSED_GOLDEN, &file);
    }
    let mut file = golden_file();
    file.flags.checksum = true;
    file.flags.compressed = true;
    read_and_compare_golden_file(file_names::CHECKSUM_COMPRESSED_GOLDEN, &file);
}