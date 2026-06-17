# stef-core

Raw serialization and deserialization layer for the **STEF** binary format.

## Usage

```toml
[dependencies]
stef-core = "1.0.0"
```

### Writing a file

```rust
use stef_core::File;
use stef_core::value::bitsize::BitSize;
use stef_core::value::bool::Bool;
use stef_core::value::number::Number;
use stef_core::value::string::Text;

fn main() {
    let mut file = File::new_empty();
    let root = file.get_root_mut();

    root.insert("name".into(),  Text::new("Johan".to_string()).into());
    root.insert("age".into(),   Number::uint(18, BitSize::Mini).into());
    root.insert("active".into(), Bool::new(true).into());

    let bytes = file.serialize().unwrap();
}
```

### Reading a file

```rust
use stef_core::File;

fn main() {
    let mut bytes = std::fs::read("file.stef").unwrap();
    let file = File::deserialize(&mut &bytes).unwrap();
    let root = file.get_root();
}
```

### Nullable values

```rust
use stef_core::value::number::Number;
use stef_core::value::bitsize::BitSize;

fn main() {
    let value = Number::uint(42, BitSize::Mini)
        .nullable()
        .unwrap();
}
```

### Flags

```rust
fn main() {
    let mut file = File::new_empty();
    file.flags.checksum   = true;
    file.flags.compressed = true;
}
```

### Arrays

Homogeneous arrays share a single element type byte. Heterogeneous arrays allow mixed types.

```rust
use stef_core::value::homoarray::HomoArray;
use stef_core::value::heteroarray::HeteroArray;
use stef_core::value::number::Number;

fn main() {
    // Homogeneous: all elements share the same type byte
    let pi    = Number::float_double_nullable(std::f64::consts::PI, true);
    let type_byte = pi.type_byte();
    let homo  = HomoArray::new(vec![pi.into()], type_byte);

    // Heterogeneous: each element carries its own type
    let hetero = HeteroArray::new(vec![
        Number::float_double(std::f64::consts::PI).into(),
        Number::float_single(f32::EPSILON).into(),
    ]);
}
```

### Nested records

```rust
use stef_core::value::record::Record;
use stef_core::value::number::Number;
use stef_core::value::bitsize::BitSize;

fn main() {
    let mut file = File::new_empty();
    let mut record = Record::new_empty();
    let fields = record.fields_mut().unwrap();

    fields.insert("age".into(),  Number::uint(18, BitSize::Mini).into());
    fields.insert("name".into(), stef_core::value::string::Text::new("Johan".to_string()).into());

    // Records can be nested directly into the root
    let root = file.get_root_mut();
    root.insert("person".into(), record.into());
}
```

## Format reference

See **SPEC.md** for the complete binary format specification.

## License

MIT