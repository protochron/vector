use remap::prelude::*;

#[derive(Clone, Copy, Debug)]
pub struct ToString;

impl Function for ToString {
    fn identifier(&self) -> &'static str {
        "to_string"
    }

    fn parameters(&self) -> &'static [Parameter] {
        &[Parameter {
            keyword: "value",
            accepts: |_| true,
            required: true,
        }]
    }

    fn compile(&self, mut arguments: ArgumentList) -> Result<Box<dyn Expression>> {
        let value = arguments.required("value")?.boxed();

        Ok(Box::new(ToStringFn { value }))
    }
}

#[derive(Debug, Clone)]
struct ToStringFn {
    value: Box<dyn Expression>,
}

impl ToStringFn {
    #[cfg(test)]
    fn new(value: Box<dyn Expression>) -> Self {
        Self { value }
    }
}

impl Expression for ToStringFn {
    fn execute(&self, state: &mut state::Program, object: &mut dyn Object) -> Result<Value> {
        use Value::*;

        let value = self.value.execute(state, object)?;

        match value {
            Bytes(_) => Ok(value),
            Integer(v) => Ok(v.to_string().into()),
            Float(v) => Ok(v.to_string().into()),
            Boolean(v) => Ok(v.to_string().into()),
            Timestamp(v) => Ok(v.to_string().into()),
            Regex(v) => Ok(v.to_string().into()),
            Null => Ok("".into()),
            Map(_) | Array(_) => Err("unable to convert value to string".into()),
        }
    }

    fn type_def(&self, state: &state::Compiler) -> TypeDef {
        self.value
            .type_def(state)
            .with_constraint(value::Kind::Bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use value::Kind;

    remap::test_type_def![
        boolean_infallible {
            expr: |_| ToStringFn { value: Literal::from(true).boxed() },
            def: TypeDef { kind: Kind::Bytes, ..Default::default() },
        }

        integer_infallible {
            expr: |_| ToStringFn { value: Literal::from(1).boxed() },
            def: TypeDef { kind: Kind::Bytes, ..Default::default() },
        }

        float_infallible {
            expr: |_| ToStringFn { value: Literal::from(1.0).boxed() },
            def: TypeDef { kind: Kind::Bytes, ..Default::default() },
        }

        null_infallible {
            expr: |_| ToStringFn { value: Literal::from(()).boxed() },
            def: TypeDef { kind: Kind::Bytes, ..Default::default() },
        }

        string_infallible {
            expr: |_| ToStringFn { value: Literal::from("foo").boxed() },
            def: TypeDef { kind: Kind::Bytes, ..Default::default() },
        }

        map_infallible {
            expr: |_| ToStringFn { value: map!{}.boxed() },
            def: TypeDef { kind: Kind::Bytes, ..Default::default() },
        }

        array_infallible {
            expr: |_| ToStringFn { value: array![].boxed() },
            def: TypeDef { kind: Kind::Bytes, ..Default::default() },
        }

        timestamp_infallible {
            expr: |_| ToStringFn { value: Literal::from(chrono::Utc::now()).boxed() },
            def: TypeDef { kind: Kind::Bytes, ..Default::default() },
        }

        fallible_value_without_default {
            expr: |_| ToStringFn { value: Variable::new("foo".to_owned(), None).boxed() },
            def: TypeDef {
                fallible: true,
                kind: Kind::Bytes,
                ..Default::default()
            },
        }
    ];

    #[test]
    fn to_string() {
        use crate::map;

        let cases = vec![
            (
                map!["foo": 20],
                Ok(Value::from("20")),
                ToStringFn::new(Box::new(Path::from("foo"))),
            ),
            (
                map!["foo": 20.5],
                Ok(Value::from("20.5")),
                ToStringFn::new(Box::new(Path::from("foo"))),
            ),
        ];

        let mut state = state::Program::default();

        for (object, exp, func) in cases {
            let mut object: Value = object.into();
            let got = func
                .execute(&mut state, &mut object)
                .map_err(|e| format!("{:#}", anyhow::anyhow!(e)));

            assert_eq!(got, exp);
        }
    }
}
