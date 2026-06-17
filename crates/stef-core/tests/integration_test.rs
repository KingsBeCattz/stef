use std::path::PathBuf;
use stef_core::File;
use stef_core::flags::Flags;
use stef_core::types::TopLevelRecord;
use stef_core::value::record::Record;
use stef_core::value::string::Text;
use stef_core::value::bitsize::BitSize;
use stef_core::value::number::Number;
use stef_core::value::bool::Bool;

struct User {
    name: String,
    email: String,
    age: u8,
    verified: bool,
}

fn base_path() -> PathBuf {
    let mut path = std::env::current_dir().unwrap();
    path.push("tests/files/");
    path
}

mod file_names {
    pub const EXAMPLE: &str = "example.stef";
    pub const EXAMPLE_CHECKSUM: &str = "example-checksum.stef";
    pub const EXAMPLE_COMPRESSED: &str = "example-compressed.stef";
    pub const EXAMPLE_CHECKSUM_COMPRESSED: &str = "example-checksum-compressed.stef";
}

fn create_user(user: &User) -> Record {
    let mut record = Record::new_empty();
    let fields = record.fields_mut().unwrap();

    fields.insert("name".into(), Text::new(user.name.clone()).into());
    fields.insert("email".into(), Text::new(user.email.clone()).into());
    fields.insert("age".into(), Number::uint(user.age as u64, BitSize::Mini).into());
    fields.insert("verified".into(), Bool::new(user.verified).into());

    record
}

fn write_user(file: &mut File, user: &User) {
    file.root.insert(user.name.clone(), create_user(user).into());
}

fn write_user_set(file: &mut File, users: &[User]) {
    for user in users {
        write_user(file, user);
    }
}

fn get_users() -> Vec<User> {
    vec![
        User {
            name: "Johan".into(),
            email: "johan@gmail.com".into(),
            age: 18,
            verified: true,
        },
        User {
            name: "Ana".into(),
            email: "ana.dev@outlook.com".into(),
            age: 19,
            verified: false,
        },
        User {
            name: "Carlos".into(),
            email: "carlos_123@yahoo.com".into(),
            age: 34,
            verified: true,
        },
        User {
            name: "María Fernanda".into(),
            email: "mfernanda.work@gmail.com".into(),
            age: 28,
            verified: true,
        },
        User {
            name: "Li".into(),
            email: "li@example.com".into(),
            age: 18,
            verified: false,
        },
        User {
            name: "AlexanderTheGreat".into(),
            email: "alexander.long.email.address@example.org".into(),
            age: 42,
            verified: false,
        },
        User {
            name: "".into(),
            email: "".into(),
            age: 0,
            verified: false,
        },
        User {
            name: "Max".into(),
            email: "max@test.dev".into(),
            age: u8::MAX,
            verified: true,
        },
    ]
}

fn example_file() -> File {
    let mut file = File::new_empty();
    let users = get_users();
    write_user_set(&mut file, &users);
    let mut meta = file.meta.unwrap_or(TopLevelRecord::new());

    meta.insert("name".into(), Text::new("Example Users Dataset".to_string()).into());
    let ts: u64 = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis() as u64;
    meta.insert("created_at".into(), Number::uint(ts, BitSize::Double).into());

    file.meta = Some(meta);
    file
}

fn file_with_checksum(file: &File) -> File {
    let flags = Flags {
        checksum: true,
        ..file.flags
    };
    File {
        flags,
        ..file.clone()
    }
}

fn file_with_compression(file: &File) -> File {
    let flags = Flags {
        compressed: true,
        ..file.flags
    };
    File {
        flags,
        ..file.clone()
    }
}

#[test]
fn serialize_deserialize_file() {
    let file = example_file();
    let serialized = file.serialize().unwrap();
    let deserialized = File::deserialize(&mut &serialized[..]).unwrap();
    assert_eq!(file, deserialized);
}

#[test]
fn serialize_deserialize_file_with_checksum() {
    let file = file_with_checksum(&example_file());
    let serialized = file.serialize().unwrap();
    let deserialized = File::deserialize(&mut &serialized[..]).unwrap();
    assert_eq!(file, deserialized);
}

#[test]
fn serialize_deserialize_file_with_compression() {
    let file = file_with_compression(&example_file());
    let serialized = file.serialize().unwrap();
    let deserialized = File::deserialize(&mut &serialized[..]).unwrap();
    assert_eq!(file, deserialized);
}

#[test]
fn serialize_deserialize_file_with_checksum_and_compression() {
    let file = file_with_compression(&file_with_checksum(&example_file()));
    let serialized = file.serialize().unwrap();
    let deserialized = File::deserialize(&mut &serialized[..]).unwrap();
    assert_eq!(file, deserialized);
}

#[test]
fn serialize_deserialize_roundtrip() {
    serialize_deserialize_file();
    serialize_deserialize_file_with_checksum();
    serialize_deserialize_file_with_compression();
    serialize_deserialize_file_with_checksum_and_compression();
}

#[test]
fn write_example_file() {
    let file = example_file();
    let serialized = file.serialize().unwrap();
    std::fs::write(base_path().join(file_names::EXAMPLE), &serialized).unwrap();
}

#[test]
fn write_example_file_with_checksum() {
    let file = file_with_checksum(&example_file());
    let serialized = file.serialize().unwrap();
    std::fs::write(base_path().join(file_names::EXAMPLE_CHECKSUM), &serialized).unwrap();
}

#[test]
fn write_example_file_with_compression() {
    let file = file_with_compression(&example_file());
    let serialized = file.serialize().unwrap();
    std::fs::write(base_path().join(file_names::EXAMPLE_COMPRESSED), &serialized).unwrap();
}

#[test]
fn write_example_file_with_checksum_and_compression() {
    let file = file_with_compression(&file_with_checksum(&example_file()));
    let serialized = file.serialize().unwrap();
    std::fs::write(base_path().join(file_names::EXAMPLE_CHECKSUM_COMPRESSED), &serialized).unwrap();
}

#[test]
fn write_example_file_roundtrip() {
    write_example_file();
    write_example_file_with_checksum();
    write_example_file_with_compression();
    write_example_file_with_checksum_and_compression();
}

#[test]
fn compare_files() {
    let example = std::fs::read(base_path().join(file_names::EXAMPLE)).unwrap();
    let example_with_checksum = std::fs::read(base_path().join(file_names::EXAMPLE_CHECKSUM)).unwrap();
    let example_with_compression = std::fs::read(base_path().join(file_names::EXAMPLE_COMPRESSED)).unwrap();
    let compress_ratio = example_with_compression.len() as f32 / example.len() as f32;
    let example_with_checksum_and_compression = std::fs::read(base_path().join(file_names::EXAMPLE_CHECKSUM_COMPRESSED)).unwrap();
    let checksum_compress_ratio = example_with_compression.len() as f32 / example.len() as f32;

    println!("{} size: {}B", file_names::EXAMPLE, example.len());
    println!("{} size: {}B", file_names::EXAMPLE_CHECKSUM, example_with_checksum.len());
    println!("{} size: {}B", file_names::EXAMPLE_COMPRESSED, example_with_compression.len());
    println!("{} size: {}B", file_names::EXAMPLE_CHECKSUM_COMPRESSED, example_with_checksum_and_compression.len());
    println!("Compression ratio: {:.2}% of {}", compress_ratio * 100.0, file_names::EXAMPLE);
    println!("Checksum compression ratio: {:.2}% of {}", checksum_compress_ratio * 100.0, file_names::EXAMPLE_CHECKSUM);
}

#[test]
fn write_and_compare_roundtrip() {
    write_example_file_roundtrip();
    compare_files();
}