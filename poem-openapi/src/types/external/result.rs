use std::borrow::Cow;

use serde_json::Value;

use crate::{
    registry::{MetaDiscriminatorObject, MetaSchema, MetaSchemaRef, Registry},
    types::{ParseError, ParseFromJSON, ParseResult, ToJSON, Type},
};

impl<T: Type, E: Type> Type for Result<T, E> {
    const IS_REQUIRED: bool = false;

    type RawValueType = Self;

    type RawElementValueType = Self;

    fn name() -> Cow<'static, str> {
        format!("result<{}, {}>", T::name(), E::name()).into()
    }

    fn schema_ref() -> MetaSchemaRef {
        MetaSchemaRef::Inline(Box::new({
            MetaSchema {
                rust_typename: Some("union::without_discriminator::Result"),
                ty: "object",
                discriminator: None,
                any_of: vec![
                    T::schema_ref().merge(MetaSchema {
                        properties: vec![("ok", T::schema_ref())],
                        ..MetaSchema::ANY
                    }),
                    E::schema_ref().merge(MetaSchema {
                        properties: vec![("err", E::schema_ref())],
                        ..MetaSchema::ANY
                    }),
                ],
                ..MetaSchema::ANY
            }
        }))
    }

    fn register(registry: &mut Registry) {
        T::register(registry);
        E::register(registry);
    }

    fn as_raw_value(&self) -> Option<&Self::RawValueType> {
        Some(self)
    }

    fn raw_element_iter<'a>(
        &'a self,
    ) -> Box<dyn Iterator<Item = &'a Self::RawElementValueType> + 'a> {
        Box::new(self.as_raw_value().into_iter())
    }
}

impl<T: ParseFromJSON, E: ParseFromJSON> ParseFromJSON for Result<T, E> {
    fn parse_from_json(value: Option<Value>) -> ParseResult<Self> {
        let mut value = value.ok_or(ParseError::expected_input())?;
        let json_map = value
            .as_object_mut()
            .ok_or(ParseError::custom("expected an object"))?;

        if let Some(ok_value) = json_map.remove("ok") {
            T::parse_from_json(Some(ok_value))
                .map(Result::Ok)
                .map_err(|error| ParseError::from(error.into_message()))
        } else if let Some(err_value) = json_map.remove("err") {
            E::parse_from_json(Some(err_value))
                .map(Result::Err)
                .map_err(|error| ParseError::from(error.into_message()))
        } else {
            Err(ParseError::expected_type(value))
        }
    }
}

impl<T: ToJSON, E: ToJSON> ToJSON for Result<T, E> {
    fn to_json(&self) -> Option<Value> {
        match self {
            Ok(t) => Some(serde_json::json!({"ok": t.to_json()})),
            Err(e) => Some(serde_json::json!({"err": e.to_json()})),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ok_mock() -> Result<usize, String> {
        Ok(10)
    }

    fn ok_json() -> serde_json::Value {
        serde_json::json!({
            "ok": 10
        })
    }

    #[test]
    fn serializes_ok_to_json() {
        assert_eq!(ok_json(), ok_mock().to_json().unwrap())
    }

    #[test]
    fn deserializes_json_ok() {
        assert_eq!(
            ok_mock(),
            Result::<usize, String>::parse_from_json(Some(ok_json())).unwrap()
        )
    }

    fn err_mock() -> Result<usize, String> {
        Err("invalid".to_string())
    }

    fn err_json() -> serde_json::Value {
        serde_json::json!({
            "err": "invalid"
        })
    }

    #[test]
    fn serializes_err_to_json() {
        assert_eq!(err_json(), err_mock().to_json().unwrap())
    }

    #[test]
    fn deserializes_json_err() {
        assert_eq!(
            err_mock(),
            Result::<usize, String>::parse_from_json(Some(err_json())).unwrap()
        )
    }
}
