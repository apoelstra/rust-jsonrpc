// Rust JSON-RPC Library
// Written in 2019 by
//   Andrew Poelstra <apoelstra@wpsoftware.net>
//
// To the extent possible under law, the author(s) have dedicated all
// copyright and related and neighboring rights to this software to
// the public domain worldwide. This software is distributed without
// any warranty.
//
// You should have received a copy of the CC0 Public Domain Dedication
// along with this software.
// If not, see <http://creativecommons.org/publicdomain/zero/1.0/>.
//

use std::borrow::Cow;
use std::hash::{Hash, Hasher};

use serde_json::Value;

/// Newtype around `Value` which allows hashing for use as hashmap keys
/// This is needed for batch requests.
///
/// The reason `Value` does not support `Hash` or `Eq` by itself
/// is that it supports `f64` values; but for batch requests we
/// will only be hashing the "id" field of the request/response
/// pair, which should never need decimal precision and therefore
/// never use `f64`.
#[derive(Clone, PartialEq, Debug)]
pub struct HashableValue<'a>(pub Cow<'a, Value>);

impl<'a> Eq for HashableValue<'a> {}

impl<'a> Hash for HashableValue<'a> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match *self.0.as_ref() {
            Value::Null => "null".hash(state),
            Value::Bool(false) => "false".hash(state),
            Value::Bool(true) => "true".hash(state),
            Value::Number(ref n) => {
                "number".hash(state);
                if let Some(n) = n.as_i64() {
                    n.hash(state);
                } else if let Some(n) = n.as_u64() {
                    n.hash(state);
                } else {
                    n.to_string().hash(state);
                }
            }
            Value::String(ref s) => {
                "string".hash(state);
                s.hash(state);
            }
            Value::Array(ref v) => {
                "array".hash(state);
                v.len().hash(state);
                for obj in v {
                    HashableValue(Cow::Borrowed(obj)).hash(state);
                }
            }
            Value::Object(ref m) => {
                "object".hash(state);
                m.len().hash(state);
                for (key, val) in m {
                    key.hash(state);
                    HashableValue(Cow::Borrowed(val)).hash(state);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;
    use std::collections::HashSet;
    use std::str::FromStr;

    use super::*;

    #[test]
    fn hash_value() {
        let val = HashableValue(Cow::Owned(Value::from_str("null").unwrap()));
        let t = HashableValue(Cow::Owned(Value::from_str("true").unwrap()));
        let f = HashableValue(Cow::Owned(Value::from_str("false").unwrap()));
        let ns =
            HashableValue(Cow::Owned(Value::from_str("[0, -0, 123.4567, -100000000]").unwrap()));
        let m =
            HashableValue(Cow::Owned(Value::from_str("{ \"field\": 0, \"field\": -0 }").unwrap()));

        let mut coll = HashSet::new();

        assert!(!coll.contains(&val));
        coll.insert(val.clone());
        assert!(coll.contains(&val));

        assert!(!coll.contains(&t));
        assert!(!coll.contains(&f));
        coll.insert(t.clone());
        assert!(coll.contains(&t));
        assert!(!coll.contains(&f));
        coll.insert(f.clone());
        assert!(coll.contains(&t));
        assert!(coll.contains(&f));

        assert!(!coll.contains(&ns));
        coll.insert(ns.clone());
        assert!(coll.contains(&ns));

        assert!(!coll.contains(&m));
        coll.insert(m.clone());
        assert!(coll.contains(&m));
    }
}
