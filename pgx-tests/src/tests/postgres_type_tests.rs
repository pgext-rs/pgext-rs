/*
Portions Copyright 2019-2021 ZomboDB, LLC.
Portions Copyright 2021-2022 Technology Concepts & Design, Inc. <support@tcdi.com>

All rights reserved.

Use of this source code is governed by the MIT license that can be found in the LICENSE file.
*/
use pgx::cstr_core::CStr;
use pgx::prelude::*;
use pgx::{InOutFuncs, PgVarlena, PgVarlenaInOutFuncs, Serializer, StringInfo};
use serde::{Deserialize, Serialize};
use std::io::Write;
use std::str::FromStr;

#[derive(Copy, Clone, PostgresType)]
#[pgvarlena_inoutfuncs]
pub struct VarlenaType {
    a: f32,
    b: f32,
    c: i64,
}

impl PgVarlenaInOutFuncs for VarlenaType {
    fn input(input: &CStr) -> PgVarlena<Self> where {
        let mut iter = input.to_str().unwrap().split(',');
        let (a, b, c) = (iter.next(), iter.next(), iter.next());

        let mut result = PgVarlena::<VarlenaType>::new();
        result.a = f32::from_str(a.unwrap()).expect("a is not a valid f32");
        result.b = f32::from_str(b.unwrap()).expect("b is not a valid f32");
        result.c = i64::from_str(c.unwrap()).expect("c is not a valid i64");
        result
    }

    fn output(&self, buffer: &mut StringInfo) {
        buffer.push_str(&format!("{},{},{}", self.a, self.b, self.c))
    }
}

#[derive(Copy, Clone, PostgresType)]
#[pgvarlena_inoutfuncs]
pub enum VarlenaEnumType {
    A,
    B,
}

impl PgVarlenaInOutFuncs for VarlenaEnumType {
    fn input(input: &CStr) -> PgVarlena<Self> where {
        let mut result = PgVarlena::<Self>::new();
        let s = input.to_str().unwrap();
        match s {
            "A" => {
                *result = VarlenaEnumType::A;
            }
            "B" => {
                *result = VarlenaEnumType::B;
            }
            _ => panic!("unexpected input"),
        }
        result
    }

    fn output(&self, buffer: &mut StringInfo) {
        let s = match self {
            VarlenaEnumType::A => "A",
            VarlenaEnumType::B => "B ",
        };
        buffer.push_str(s)
    }
}

#[derive(Serialize, Deserialize, PostgresType)]
#[inoutfuncs]
pub struct CustomTextFormatSerializedType {
    a: f32,
    b: f32,
    c: i64,
}

impl InOutFuncs for CustomTextFormatSerializedType {
    fn input(input: &CStr) -> Self {
        let mut iter = input.to_str().unwrap().split(',');
        let (a, b, c) = (iter.next(), iter.next(), iter.next());

        CustomTextFormatSerializedType {
            a: f32::from_str(a.unwrap()).expect("a is not a valid f32"),
            b: f32::from_str(b.unwrap()).expect("b is not a valid f32"),
            c: i64::from_str(c.unwrap()).expect("c is not a valid i64"),
        }
    }

    fn output(&self, buffer: &mut StringInfo) {
        buffer.push_str(&format!("{},{},{}", self.a, self.b, self.c))
    }
}

#[pg_extern(immutable)]
fn fn_takes_option(input: Option<CustomTextFormatSerializedType>) -> String {
    input.map_or("nothing".to_string(), |c| {
        let mut b = StringInfo::new();
        c.output(&mut b);
        b.to_string()
    })
}

#[derive(Serialize, Deserialize, PostgresType)]
#[inoutfuncs]
pub enum CustomTextFormatSerializedEnumType {
    A,
    B,
}

impl InOutFuncs for CustomTextFormatSerializedEnumType {
    fn input(input: &CStr) -> Self {
        let s = input.to_str().unwrap();
        match s {
            "A" => CustomTextFormatSerializedEnumType::A,
            "B" => CustomTextFormatSerializedEnumType::B,
            _ => panic!("unexpected input"),
        }
    }

    fn output(&self, buffer: &mut StringInfo) {
        let s = match self {
            CustomTextFormatSerializedEnumType::A => "A",
            CustomTextFormatSerializedEnumType::B => "B",
        };
        buffer.push_str(s)
    }
}

#[pg_extern(immutable)]
fn fn_takes_option_enum(input: Option<CustomTextFormatSerializedEnumType>) -> String {
    input.map_or("nothing".to_string(), |c| {
        let mut b = StringInfo::new();
        c.output(&mut b);
        b.to_string()
    })
}

#[derive(Serialize, Deserialize, PostgresType)]
pub struct JsonType {
    a: f32,
    b: f32,
    c: i64,
}

#[derive(Serialize, Deserialize, PostgresType)]
#[serde(tag = "type")]
pub enum JsonEnumType {
    E1 { a: f32 },
    E2 { b: f32 },
}

#[derive(Serialize, Deserialize, PostgresType)]
#[custom_serializer]
pub struct CustomSerialized;

impl<'de> Serializer<'de> for CustomSerialized {
    fn to_writer<W: Write>(&self, mut writer: W) {
        writer.write(&[1]).expect("can't write");
    }

    fn from_slice(slice: &'de [u8]) -> Self {
        if slice != &[1] {
            panic!("wrong type")
        } else {
            CustomSerialized
        }
    }
}

#[derive(Serialize, Deserialize, PostgresType)]
#[custom_serializer]
pub struct AnotherCustomSerialized;

impl<'de> Serializer<'de> for AnotherCustomSerialized {
    fn to_writer<W: Write>(&self, mut writer: W) {
        writer.write(&[0]).expect("can't write");
    }

    fn from_slice(slice: &'de [u8]) -> Self {
        if slice != &[0] {
            panic!("wrong type")
        } else {
            AnotherCustomSerialized
        }
    }
}

#[cfg(any(test, feature = "pg_test"))]
#[pgx::pg_schema]
mod tests {
    #[allow(unused_imports)]
    use crate as pgx_tests;

    use crate::tests::postgres_type_tests::{
        AnotherCustomSerialized, CustomSerialized, CustomTextFormatSerializedEnumType,
        CustomTextFormatSerializedType, JsonEnumType, JsonType, VarlenaEnumType, VarlenaType,
    };
    use pgx::prelude::*;
    use pgx::PgVarlena;

    #[pg_test]
    fn test_mytype() {
        let result = Spi::get_one::<PgVarlena<VarlenaType>>("SELECT '1.0,2.0,3'::VarlenaType")
            .expect("SPI returned NULL");
        assert_eq!(result.a, 1.0);
        assert_eq!(result.b, 2.0);
        assert_eq!(result.c, 3);
    }

    #[pg_test]
    fn test_my_enum_type() {
        let result = Spi::get_one::<PgVarlena<VarlenaEnumType>>("SELECT 'B'::VarlenaEnumType")
            .expect("SPI returned NULL");
        assert!(matches!(*result, VarlenaEnumType::B));
    }

    #[pg_test]
    fn test_call_with_value() {
        let result = Spi::get_one::<String>(
            "SELECT fn_takes_option('1.0,2.0,3'::CustomTextFormatSerializedType);",
        );
        assert_eq!("1,2,3", result.unwrap());
    }

    #[pg_test]
    fn test_call_with_null() {
        let result = Spi::get_one::<String>("SELECT fn_takes_option(NULL);");
        assert_eq!(Some(String::from("nothing")), result);
    }

    #[pg_test]
    fn test_serializedtype() {
        let result = Spi::get_one::<CustomTextFormatSerializedType>(
            "SELECT '1.0,2.0,3'::CustomTextFormatSerializedType",
        )
        .expect("SPI returned NULL");
        assert_eq!(result.a, 1.0);
        assert_eq!(result.b, 2.0);
        assert_eq!(result.c, 3);
    }

    #[pg_test]
    fn test_call_with_enum_value() {
        let result = Spi::get_one::<String>(
            "SELECT fn_takes_option_enum('A'::CustomTextFormatSerializedEnumType);",
        );
        assert_eq!("A", result.unwrap());
    }

    #[pg_test]
    fn test_call_with_enum_null() {
        let result = Spi::get_one::<String>("SELECT fn_takes_option_enum(NULL);");
        assert_eq!(Some(String::from("nothing")), result);
    }

    #[pg_test]
    fn test_serialized_enum_type() {
        let result = Spi::get_one::<CustomTextFormatSerializedEnumType>(
            "SELECT 'B'::CustomTextFormatSerializedEnumType",
        )
        .expect("SPI returned NULL");

        assert!(matches!(result, CustomTextFormatSerializedEnumType::B));
    }

    #[pg_test]
    fn test_jsontype() {
        let result = Spi::get_one::<JsonType>(r#"SELECT '{"a": 1.0, "b": 2.0, "c": 3}'::JsonType"#)
            .expect("SPI returned NULL");
        assert_eq!(result.a, 1.0);
        assert_eq!(result.b, 2.0);
        assert_eq!(result.c, 3);
    }

    #[pg_test]
    fn test_json_enum_type() {
        let result =
            Spi::get_one::<JsonEnumType>(r#"SELECT '{"type": "E1", "a": 1.0}'::JsonEnumType"#)
                .expect("SPI returned NULL");
        assert!(matches!(result, JsonEnumType::E1 { a } if a == 1.0));
    }

    #[pg_test]
    fn custom_serializer() {
        let s = CustomSerialized;
        let _ = Spi::get_one_with_args::<CustomSerialized>(
            r#"SELECT $1"#,
            vec![(PgOid::Custom(CustomSerialized::type_oid()), s.into_datum())],
        )
        .unwrap();
    }

    #[pg_test(error = "wrong type")]
    fn custom_serializer_wrong_type() {
        let s = CustomSerialized;
        let _ = Spi::get_one_with_args::<AnotherCustomSerialized>(
            r#"SELECT $1"#,
            vec![(PgOid::Custom(CustomSerialized::type_oid()), s.into_datum())],
        )
        .unwrap();
    }
}
