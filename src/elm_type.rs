use std::{convert::identity, sync::LazyLock};

use crate::error::Result;
use serde::Deserialize;
use serde_reflection::{ContainerFormat, Format, Named, Registry, Samples};

static SAMPLES: LazyLock<Samples> = LazyLock::new(Samples::new);

pub fn convert<'de, T: Deserialize<'de>>(
    format_adjustment: impl Fn(String) -> String,
) -> Result<String> {
    let mut tracer = serde_reflection::Tracer::new(Default::default());
    let (format, _) = tracer.trace_type_once::<T>(&SAMPLES)?;
    let registry = tracer.registry()?;

    Ok(convert_format(format, &registry, format_adjustment))
}

fn convert_format(
    format: Format,
    registry: &Registry,
    format_adjustment: impl Fn(String) -> String,
) -> String {
    match format {
        Format::Variable(_) => panic!("Unknown format"),
        Format::TypeName(type_name) => {
            let referenced_format = registry.get(&type_name).unwrap();
            match referenced_format {
                ContainerFormat::UnitStruct => "()".to_owned(),
                ContainerFormat::NewTypeStruct(inner) => {
                    convert_format(*inner.clone(), registry, format_adjustment)
                }
                ContainerFormat::TupleStruct(vec) => convert_tuple_format(vec.clone(), registry),
                ContainerFormat::Struct(vec) => convert_struct_format(vec.clone(), registry),
                ContainerFormat::Enum(_) => todo!("Enums are not supported"),
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
        Format::Option(inner) => format_adjustment(format!(
            "Maybe {}",
            convert_format(*inner, registry, wrap_in_round_brackets)
        )),
        Format::Seq(inner) => format_adjustment(format!(
            "List {}",
            convert_format(*inner, registry, wrap_in_round_brackets)
        )),
        Format::Map { key, value } => format_adjustment(format!(
            "Dict {} {}",
            convert_format(*key, registry, wrap_in_round_brackets),
            convert_format(*value, registry, wrap_in_round_brackets)
        )),
        Format::Tuple(vec) => convert_tuple_format(vec, registry),
        Format::TupleArray { content, size: _ } => {
            format!(
                "List {}",
                convert_format(*content, registry, wrap_in_round_brackets)
            )
        }
    }
}

fn convert_tuple_format(vec: Vec<Format>, registry: &Registry) -> String {
    let types = vec
        .into_iter()
        .map(|inner| convert_format(inner, registry, identity))
        .collect::<Vec<_>>()
        .join(", ");
    format!("( {types} )")
}

pub fn wrap_in_round_brackets(str: String) -> String {
    format!("({str})")
}

fn convert_struct_format(vec: Vec<Named<Format>>, registry: &Registry) -> String {
    let types = vec
        .into_iter()
        .map(|inner| {
            format!(
                "{} : {}",
                inner.name,
                convert_format(inner.value, registry, identity)
            )
        })
        .collect::<Vec<_>>()
        .join(", ");
    format!("{{ {types} }}")
}

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, convert::identity};

    use serde::Deserialize;

    use crate::elm_type::wrap_in_round_brackets;

    use super::convert;

    #[test]
    fn simple_struct() {
        #[derive(Deserialize, Debug)]
        #[allow(dead_code)]
        struct Test {
            a: i64,
            b: bool,
        }
        assert_eq!(convert::<Test>(identity).unwrap(), "{ a : Int, b : Bool }");
    }

    #[test]
    fn simple_list() {
        assert_eq!(convert::<Vec<i8>>(identity).unwrap(), "List Int");
    }

    #[test]
    fn simple_tuple() {
        assert_eq!(
            convert::<(i16, String)>(identity).unwrap(),
            "( Int, String )"
        );
    }

    #[test]
    fn simple_option() {
        assert_eq!(convert::<Option<char>>(identity).unwrap(), "Maybe Char");
    }

    #[test]
    fn format_adjustment_option() {
        assert_eq!(
            convert::<Option<char>>(wrap_in_round_brackets).unwrap(),
            "(Maybe Char)"
        );
    }

    #[test]
    fn format_adjustment_vec() {
        assert_eq!(
            convert::<Vec<char>>(wrap_in_round_brackets).unwrap(),
            "(List Char)"
        );
    }

    #[test]
    fn simple_map() {
        assert_eq!(
            convert::<HashMap<String, u16>>(identity).unwrap(),
            "Dict String Int"
        );
    }

    #[test]
    fn nested_option() {
        assert_eq!(
            convert::<Option<Option<u8>>>(identity).unwrap(),
            "Maybe (Maybe Int)"
        );
    }

    #[test]
    fn nested_struct() {
        #[derive(Deserialize, Debug)]
        #[allow(dead_code)]
        struct Test {
            a: i64,
            b: bool,
        }
        #[derive(Deserialize, Debug)]
        #[allow(dead_code)]
        struct Test2 {
            c: Test,
            d: Vec<Test>,
        }
        assert_eq!(
            convert::<Option<Test2>>(identity).unwrap(),
            "Maybe { c : { a : Int, b : Bool }, d : List { a : Int, b : Bool } }"
        );
    }
}
