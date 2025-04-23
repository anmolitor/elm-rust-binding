#[cfg(feature = "v8")]
mod v8 {
    use elm_rust_binding::{ElmRoot, Result};
    use serde::{Deserialize, Serialize};

    macro_rules! add5_test {
        ($int_type:ty) => {{
            let elm_root = ElmRoot::new("./tests/elm/src")?;
            let elm_add5 = elm_root.prepare("Test.add5")?;
            let result: $int_type = elm_add5.call(1)?;
            assert_eq!(result, 6);
            Ok(())
        }};
    }

    #[test]
    fn i8() -> Result<()> {
        add5_test!(i8)
    }

    #[test]
    fn i16() -> Result<()> {
        add5_test!(i16)
    }

    #[test]
    fn i32() -> Result<()> {
        add5_test!(i32)
    }

    #[test]
    fn i64() -> Result<()> {
        add5_test!(i64)
    }

    #[test]
    fn u8() -> Result<()> {
        add5_test!(u8)
    }

    #[test]
    fn u16() -> Result<()> {
        add5_test!(u16)
    }

    #[test]
    fn u32() -> Result<()> {
        add5_test!(u32)
    }

    #[test]
    fn u64() -> Result<()> {
        add5_test!(u64)
    }

    #[test]
    fn string() -> Result<()> {
        let elm_root = ElmRoot::new("./tests/elm/src")?;
        let elm_prepend_test = elm_root.prepare("Test.prependTest")?;
        let result: String = elm_prepend_test.call("abc".to_owned())?;
        assert_eq!(result, "testabc");
        Ok(())
    }

    #[test]
    fn structs() -> Result<()> {
        #[derive(Serialize, Deserialize)]
        struct StructIn {
            a: Option<i32>,
            b: Vec<bool>,
        }
        #[derive(Deserialize, PartialEq, Eq, Debug)]
        struct StructOut {
            c: Vec<i32>,
            d: Option<bool>,
        }

        let elm_root = ElmRoot::new("./tests/elm/src")?;
        let elm_some_struct_mapper = elm_root.prepare("Test.someStructMapper")?;
        let result: Vec<StructOut> = elm_some_struct_mapper.call(vec![StructIn {
            a: Some(5),
            b: vec![true, false],
        }])?;
        assert_eq!(
            result,
            vec![StructOut {
                c: vec![5],
                d: Some(true)
            }]
        );
        Ok(())
    }
}

#[cfg(feature = "quickjs")]
mod quickjs {
    use elm_rust_binding::{ElmRoot, Result};
    use serde::{Deserialize, Serialize};

    macro_rules! add5_test {
        ($int_type:ty) => {{
            let elm_root = ElmRoot::new("./tests/elm/src")?;
            let elm_add5 = elm_root.prepare("Test.add5").await?;
            let result: $int_type = elm_add5.call(1).await?;
            assert_eq!(result, 6);
            Ok(())
        }};
    }

    #[tokio::test]
    async fn i8() -> Result<()> {
        add5_test!(i8)
    }

    #[tokio::test]
    async fn i16() -> Result<()> {
        add5_test!(i16)
    }

    #[tokio::test]
    async fn i32() -> Result<()> {
        add5_test!(i32)
    }

    #[tokio::test]
    async fn i64() -> Result<()> {
        add5_test!(i64)
    }

    #[tokio::test]
    async fn u8() -> Result<()> {
        add5_test!(u8)
    }

    #[tokio::test]
    async fn u16() -> Result<()> {
        add5_test!(u16)
    }

    #[tokio::test]
    async fn u32() -> Result<()> {
        add5_test!(u32)
    }

    #[tokio::test]
    async fn u64() -> Result<()> {
        add5_test!(u64)
    }

    #[tokio::test]
    async fn string() -> Result<()> {
        let elm_root = ElmRoot::new("./tests/elm/src")?;
        let elm_prepend_test = elm_root.prepare("Test.prependTest").await?;
        let result: String = elm_prepend_test.call("abc".to_owned()).await?;
        assert_eq!(result, "testabc");
        Ok(())
    }

    #[tokio::test]
    async fn structs() -> Result<()> {
        #[derive(Serialize, Deserialize)]
        struct StructIn {
            a: Option<i32>,
            b: Vec<bool>,
        }
        #[derive(Deserialize, PartialEq, Eq, Debug)]
        struct StructOut {
            c: Vec<i32>,
            d: Option<bool>,
        }

        let elm_root = ElmRoot::new("./tests/elm/src")?;
        let elm_some_struct_mapper = elm_root.prepare("Test.someStructMapper").await?;
        let result: Vec<StructOut> = elm_some_struct_mapper
            .call(vec![StructIn {
                a: Some(5),
                b: vec![true, false],
            }])
            .await?;
        assert_eq!(
            result,
            vec![StructOut {
                c: vec![5],
                d: Some(true)
            }]
        );
        Ok(())
    }
}
