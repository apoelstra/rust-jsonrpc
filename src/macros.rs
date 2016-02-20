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
    ($name:ident, $($fe:ident $(<- $alt:expr)*),*) => (
        impl ::serde::Deserialize for $name {
            fn deserialize<D>(deserializer: &mut D) -> Result<$name, D::Error>
                where D: serde::de::Deserializer
            {
                // begin type defs
                #[allow(non_camel_case_types)]
                enum Enum { $($fe),* }

                struct EnumVisitor;
                impl ::serde::de::Visitor for EnumVisitor {
                    type Value = Enum;

                    fn visit_str<E>(&mut self, value: &str) -> Result<Enum, E>
                        where E: ::serde::de::Error
                    {
                        match value {
                            $(
                            stringify!($fe) => Ok(Enum::$fe)
                            $(, $alt => Ok(Enum::$fe))*
                            ),*,
                            _ => Err(::serde::de::Error::syntax("unexpected field")),
                        }
                    }
                }

                impl ::serde::Deserialize for Enum {
                    fn deserialize<D>(deserializer: &mut D) -> Result<Enum, D::Error>
                        where D: ::serde::de::Deserializer
                    {
                        deserializer.visit_str(EnumVisitor)
                    }
                }

                struct Visitor;

                impl ::serde::de::Visitor for Visitor {
                    type Value = $name;

                    fn visit_map<V>(&mut self, mut v: V) -> Result<$name, V::Error>
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
                        Ok($name{ $($fe: $fe),* })
                    }
                }
                // end type defs

                static FIELDS: &'static [&'static str] = &[$(stringify!($fe)),*];

                deserializer.visit_struct(stringify!($name), FIELDS, Visitor)
            }
        }

        impl ::serde::Serialize for $name {
            fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
                where S: ::serde::Serializer
            {
                // begin type defs
                #[repr(u16)]
                #[derive(Copy, Clone)]
                #[allow(non_camel_case_types)]
                #[allow(dead_code)]
                enum State { $($fe),* , Finished }

                struct MapVisitor<'a> {
                    value: &'a $name,
                    state: State,
                }

                impl<'a> ::serde::ser::MapVisitor for MapVisitor<'a> {
                    fn visit<S>(&mut self, serializer: &mut S) -> Result<Option<()>, S::Error>
                        where S: ::serde::Serializer
                    {
                        match self.state {
                            $(State::$fe => {
                                self.state = unsafe { ::std::mem::transmute(self.state as u16 + 1) };
                                // Use the last alternate name for serialization; in the common case
                                // with zero or one alternates this does the RIght Thing
                                let names = [stringify!($fe), $($alt),*];
                                Ok(Some(try!(serializer.visit_struct_elt(names[names.len() - 1], &self.value.$fe))))
                            })*
                            State::Finished => {
                                Ok(None)
                            }
                        }
                    }
                }
                // end type defs

                serializer.visit_struct(stringify!($name), MapVisitor {
                    value: self,
                    state: unsafe { ::std::mem::transmute(0u16) },
                })
            }
        }
    )
}

#[macro_export]
macro_rules! serde_struct_enum_impl {
    ($name:ident,
     $($varname:ident, $structname:ident, $($fe:ident $(<- $alt:expr)*),*);*
    ) => (
        impl ::serde::Deserialize for $name {
            fn deserialize<D>(deserializer: &mut D) -> Result<$name, D::Error>
                where D: serde::de::Deserializer
            {
                // start type defs
                $(#[allow(non_camel_case_types)] enum $varname { $($fe),* })*
                enum Enum { $($varname($varname)),* }

                struct EnumVisitor;
                impl ::serde::de::Visitor for EnumVisitor {
                    type Value = Enum;

                    fn visit_str<E>(&mut self, value: &str) -> Result<Enum, E>
                        where E: ::serde::de::Error
                    {
                        $($(
                        if value == stringify!($fe) $(|| value == $alt)* {
                            Ok(Enum::$varname($varname::$fe))
                        } else)*)* {
                            Err(::serde::de::Error::syntax("unexpected field"))
                        }
                    }
                }

                impl ::serde::Deserialize for Enum {
                    fn deserialize<D>(deserializer: &mut D) -> Result<Enum, D::Error>
                        where D: ::serde::de::Deserializer
                    {
                        deserializer.visit_str(EnumVisitor)
                    }
                }

                struct Visitor;

                impl ::serde::de::Visitor for Visitor {
                    type Value = $name;

                    #[allow(non_snake_case)] //for $structname
                    #[allow(unused_assignments)] // for `$fe = None` hack
                    fn visit_map<V>(&mut self, mut v: V) -> Result<$name, V::Error>
                        where V: ::serde::de::MapVisitor
                    {
                        $(
                        $(let mut $fe = None;)*
                        // In case of multiple variants having the same field, some of
                        // the above lets will get shadowed. We therefore need to tell
                        // rustc its type, since it otherwise cannot infer it, causing
                        // a compilation error. Hence this hack, which the denizens of
                        // #rust and I had a good laugh over:
                        if false { let _ = $structname { $($fe: $fe.unwrap()),* }; }
                        // The above expression moved $fe so we have to reassign it :)
                        $($fe = None;)*
                        )*

                        loop {
                            match try!(v.visit_key()) {
                                $($(Some(Enum::$varname($varname::$fe)) => {
                                    $fe = Some(try!(v.visit_value())); })*)*
                                None => { break; }
                            }
                        }

                        // try to find a variant for which we have all fields
                        $(
                            let mut $structname = true;
                            $(if $fe.is_none() { $structname = false })*
                            // if we found one, success. extra fields is not an error,
                            // it'd be too much of a PITA to manage overlapping field
                            // sets otherwise.
                            if $structname {
                                $(let $fe = $fe.unwrap();)*
                                try!(v.end());
                                return Ok($name::$varname($structname { $($fe: $fe),* }))
                            }
                        )*
                        // If we get here we failed
                        Err(::serde::de::Error::syntax("did not get all fields"))
                    }
                }
                // end type defs

                static FIELDS: &'static [&'static str] = &[$($(stringify!($fe)),*),*];

                deserializer.visit_struct(stringify!($name), FIELDS, Visitor)
            }
        }

        // impl Serialize (and Deserialize, tho we don't need it) for the underlying structs
        $( serde_struct_impl!($structname, $($fe $(<- $alt)*),*); )*
        // call serialize on the right one
        impl ::serde::Serialize for $name {
            fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
                where S: ::serde::Serializer
            {
                match *self {
                    $($name::$varname(ref x) => x.serialize(serializer)),*
                }
            }
        }
    )
}

#[cfg(test)]
mod tests {
    use serde;

    struct Variant1 {
        success: bool,
        success_message: String
    }

    struct Variant2 {
        success: bool,
        errors: Vec<String>
    }

    pub enum Reply {
        Good(Variant1),
        Bad(Variant2),
    }
    serde_struct_enum_impl!(Reply,
        Good, Variant1, success, success_message;
        Bad, Variant2, success, errors
    );
}

