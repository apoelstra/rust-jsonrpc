// Rust JSON-RPC Library
// Written in 2015 by
//     Andrew Poelstra <apoelstra@wpsoftware.net>
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

//! # Macros
//!
//! Macros to replace serde's codegen while that is not stable
//!

#[macro_export]
macro_rules! serde_struct_serialize {
    ($name:ident, $mapvisitor:ident, $($fe:ident => $n:expr),*) => (
        impl ::serde::Serialize for $name {
            fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
                where S: ::serde::Serializer
            {
                struct $mapvisitor<'a> {
                    value: &'a $name,
                    state: u8,
                }

                impl<'a> ::serde::ser::MapVisitor for $mapvisitor<'a> {
                    fn visit<S>(&mut self, serializer: &mut S) -> Result<Option<()>, S::Error>
                        where S: ::serde::Serializer
                    {
                        match self.state {
                            $($n => {
                                self.state += 1;
                                Ok(Some(try!(serializer.visit_struct_elt(stringify!($fe), &self.value.$fe))))
                            })*
                            _ => {
                                Ok(None)
                            }
                        }
                    }
                }
                serializer.visit_struct(stringify!($name), $mapvisitor {
                    value: self,
                    state: 0,
                })
            }
        }
    )
}

#[macro_export]
macro_rules! serde_struct_deserialize {
    ($name:ident, $visitor:ident, $enum_ty:ident, $enum_visitor:ident, $($fe:ident => $en:ident),*) => (
        enum $enum_ty { $($en),* }

        impl ::serde::Deserialize for $enum_ty {
            fn deserialize<D>(deserializer: &mut D) -> Result<$enum_ty, D::Error>
                where D: ::serde::de::Deserializer
            {
                struct $enum_visitor;
                impl ::serde::de::Visitor for $enum_visitor {
                    type Value = $enum_ty;

                    fn visit_str<E>(&mut self, value: &str) -> Result<$enum_ty, E>
                        where E: ::serde::de::Error
                    {
                        match value {
                            $(stringify!($fe) => Ok($enum_ty::$en)),*,
                            _ => Err(::serde::de::Error::syntax("unexpected field")),
                        }
                    }
                }
                deserializer.visit($enum_visitor)
            }
        }

        impl ::serde::Deserialize for $name {
            fn deserialize<D>(deserializer: &mut D) -> Result<$name, D::Error>
                where D: serde::de::Deserializer
            {
                static FIELDS: &'static [&'static str] = &[$(stringify!($fe)),*];

                struct $visitor;
                impl ::serde::de::Visitor for $visitor {
                    type Value = $name;

                    fn visit_map<V>(&mut self, mut v: V) -> Result<$name, V::Error>
                        where V: ::serde::de::MapVisitor
                    {
                        $(let mut $fe = None;)*

                        loop {
                            match try!(v.visit_key()) {
                                $(Some($enum_ty::$en) => { $fe = Some(try!(v.visit_value())); })*
                                None => { break; }
                            }
                        }

                        $(let $fe = match $fe {
                            Some(x) => x,
                            None => try!(v.missing_field(stringify!($fe))),
                        };)*
                        try!(v.end());
                        Ok($name{ $($fe: $fe),* })
                    }
                }
                deserializer.visit_struct(stringify!($name), FIELDS, $visitor)
            }
        }

    )
}

