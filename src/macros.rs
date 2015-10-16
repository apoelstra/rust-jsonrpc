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
macro_rules! serde_struct_impl {
    ($name:ident, $modname:ident, $($fe:ident),*) => (
        mod $modname {
            #[allow(non_camel_case_types)]
            enum Enum { $($fe),* }

            struct EnumVisitor;
            impl ::serde::de::Visitor for EnumVisitor {
                type Value = Enum;

                fn visit_str<E>(&mut self, value: &str) -> Result<Enum, E>
                    where E: ::serde::de::Error
                {
                    match value {
                        $(stringify!($fe) => Ok(Enum::$fe)),*,
                        _ => Err(::serde::de::Error::syntax("unexpected field")),
                    }
                }
            }

            impl ::serde::Deserialize for Enum {
                fn deserialize<D>(deserializer: &mut D) -> Result<Enum, D::Error>
                    where D: ::serde::de::Deserializer
                {
                    deserializer.visit(EnumVisitor)
                }
            }

            pub struct Visitor;

            impl ::serde::de::Visitor for Visitor {
                type Value = super::$name;

                fn visit_map<V>(&mut self, mut v: V) -> Result<super::$name, V::Error>
                    where V: ::serde::de::MapVisitor
                {
                    $(let mut $fe = None;)*

                    loop {
                        match try!(v.visit_key()) {
                            $(Some(Enum::$fe) => { $fe = Some(try!(v.visit_value())); })*
                            None => { break; }
                        }
                    }

                    $(let $fe = match $fe {
                        Some(x) => x,
                        None => try!(v.missing_field(stringify!($fe))),
                    };)*
                    try!(v.end());
                    Ok(super::$name{ $($fe: $fe),* })
                }
            }

            #[repr(u16)]
            #[derive(Copy, Clone)]
            #[allow(non_camel_case_types)]
            #[allow(dead_code)]
            pub enum State { $($fe),* , Finished }

            pub struct MapVisitor<'a> {
                pub value: &'a super::$name,
                pub state: State,
            }

            impl<'a> ::serde::ser::MapVisitor for MapVisitor<'a> {
                fn visit<S>(&mut self, serializer: &mut S) -> Result<Option<()>, S::Error>
                    where S: ::serde::Serializer
                {
                    match self.state {
                        $(State::$fe => {
                            self.state = unsafe { ::std::mem::transmute(self.state as u16 + 1) };
                            Ok(Some(try!(serializer.visit_struct_elt(stringify!($fe), &self.value.$fe))))
                        })*
                        State::Finished => {
                            Ok(None)
                        }
                    }
                }
            }
        }


        impl ::serde::Deserialize for $name {
            fn deserialize<D>(deserializer: &mut D) -> Result<$name, D::Error>
                where D: serde::de::Deserializer
            {
                static FIELDS: &'static [&'static str] = &[$(stringify!($fe)),*];

                deserializer.visit_struct(stringify!($name), FIELDS, $modname::Visitor)
            }
        }

        impl ::serde::Serialize for $name {
            fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
                where S: ::serde::Serializer
            {
                serializer.visit_struct(stringify!($name), $modname::MapVisitor {
                    value: self,
                    state: unsafe { ::std::mem::transmute(0u16) },
                })
            }
        }
    )
}

