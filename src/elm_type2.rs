use std::{error, fmt, marker::PhantomData};

use serde::{
    de::{DeserializeOwned, DeserializeSeed, MapAccess, SeqAccess, Visitor},
    Deserialize, Deserializer,
};
use serde_reflection::{ContainerFormat, Format, Named, Registry};

pub fn convert<T: DeserializeOwned>() -> Result<String, Box<dyn std::error::Error>> {
    let mut output = String::new();

    let mut tracer = serde_reflection::Tracer::new(Default::default());
    let (format, _) = tracer.trace_type_once::<T>(&Default::default())?;
    let registry = tracer.registry()?;

    Ok(convert_format(format, &registry))
}

fn convert_format(format: Format, registry: &Registry) -> String {
    match format {
        Format::Variable(variable) => panic!("Unknown format"),
        Format::TypeName(type_name) => {
            let referenced_format = registry.get(&type_name).unwrap();
            match referenced_format {
                ContainerFormat::UnitStruct => "()".to_owned(),
                ContainerFormat::NewTypeStruct(inner) => convert_format(*inner.clone(), registry),
                ContainerFormat::TupleStruct(vec) => convert_tuple_format(vec.clone(), registry),
                ContainerFormat::Struct(vec) => convert_struct_format(vec.clone(), registry),
                ContainerFormat::Enum(btree_map) => todo!("Enums are not supported"),
            }
        }
        Format::Unit => "()".to_owned(),
        Format::Bool => "Bool".to_owned(),
        Format::I8 => "Int".to_owned(),
        Format::I16 => "Int".to_owned(),
        Format::I32 => "Int".to_owned(),
        Format::I64 => "Int".to_owned(),
        Format::I128 => "Int".to_owned(),
        Format::U8 => "Int".to_owned(),
        Format::U16 => "Int".to_owned(),
        Format::U32 => "Int".to_owned(),
        Format::U64 => "Int".to_owned(),
        Format::U128 => "Int".to_owned(),
        Format::F32 => "Float".to_owned(),
        Format::F64 => "Float".to_owned(),
        Format::Char => "Char".to_owned(),
        Format::Str => "String".to_owned(),
        Format::Bytes => "Bytes".to_owned(),
        Format::Option(inner) => format!("Some ({})", convert_format(*inner, registry)),
        Format::Seq(inner) => format!("List ({})", convert_format(*inner, registry)),
        Format::Map { key, value } => format!(
            "Dict ({}) ({})",
            convert_format(*key, registry),
            convert_format(*value, registry)
        ),
        Format::Tuple(vec) => convert_tuple_format(vec, registry),
        Format::TupleArray { content, size } => {
            format!("List ({})", convert_format(*content, registry))
        }
    }
}

fn convert_tuple_format(vec: Vec<Format>, registry: &Registry) -> String {
    let types = vec
        .into_iter()
        .map(|inner| convert_format(inner, registry))
        .collect::<Vec<_>>()
        .join(", ");
    format!("({types})")
}

fn convert_struct_format(vec: Vec<Named<Format>>, registry: &Registry) -> String {
    let types = vec
        .into_iter()
        .map(|inner| format!("{}: {}", inner.name, convert_format(inner.value, registry)))
        .collect::<Vec<_>>()
        .join(", ");
    format!("{{ {types} }}")
}

#[test]
fn test() {
    #[derive(Deserialize, Debug)]
    struct Test {
        a: i64,
        b: bool,
    }

    println!("{:?}", convert::<Test>());
}
