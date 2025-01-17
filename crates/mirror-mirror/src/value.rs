use alloc::borrow::ToOwned;
use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use core::any::Any;
use core::cmp::Ordering;
use core::fmt;

use ordered_float::OrderedFloat;

use crate::enum_::EnumValue;
use crate::struct_::StructValue;
use crate::tuple::TupleValue;
use crate::tuple_struct::TupleStructValue;
use crate::type_info::graph::NodeId;
use crate::type_info::graph::OpaqueNode;
use crate::type_info::graph::TypeGraph;
use crate::FromReflect;
use crate::Reflect;
use crate::ReflectMut;
use crate::ReflectOwned;
use crate::ReflectRef;
use crate::ScalarMut;
use crate::ScalarOwned;
use crate::ScalarRef;
use crate::TypeRoot;
use crate::Typed;

#[allow(non_camel_case_types)]
#[derive(Debug, Clone)]
#[cfg_attr(feature = "speedy", derive(speedy::Readable, speedy::Writable))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Value {
    usize(usize),
    u8(u8),
    u16(u16),
    u32(u32),
    u64(u64),
    u128(u128),
    i8(i8),
    i16(i16),
    i32(i32),
    i64(i64),
    i128(i128),
    bool(bool),
    char(char),
    f32(f32),
    f64(f64),
    String(String),
    StructValue(Box<StructValue>),
    EnumValue(Box<EnumValue>),
    TupleStructValue(TupleStructValue),
    TupleValue(TupleValue),
    List(Vec<Value>),
    Map(BTreeMap<Value, Value>),
}

impl FromReflect for Value {
    fn from_reflect(reflect: &dyn Reflect) -> Option<Self> {
        Some(reflect.to_value())
    }
}

#[allow(non_camel_case_types)]
#[derive(Eq, PartialEq, PartialOrd, Ord)]
enum OrdEqValue<'a> {
    usize(usize),
    u8(u8),
    u16(u16),
    u32(u32),
    u64(u64),
    u128(u128),
    i8(i8),
    i16(i16),
    i32(i32),
    i64(i64),
    i128(i128),
    bool(bool),
    char(char),
    f32(OrderedFloat<f32>),
    f64(OrderedFloat<f64>),
    String(&'a str),
    StructValue(&'a StructValue),
    EnumValue(&'a EnumValue),
    TupleStructValue(&'a TupleStructValue),
    TupleValue(&'a TupleValue),
    List(&'a [Value]),
    Map(&'a BTreeMap<Value, Value>),
}

impl<'a> From<&'a Value> for OrdEqValue<'a> {
    fn from(value: &'a Value) -> Self {
        match value {
            Value::usize(inner) => OrdEqValue::usize(*inner),
            Value::u8(inner) => OrdEqValue::u8(*inner),
            Value::u16(inner) => OrdEqValue::u16(*inner),
            Value::u32(inner) => OrdEqValue::u32(*inner),
            Value::u64(inner) => OrdEqValue::u64(*inner),
            Value::u128(inner) => OrdEqValue::u128(*inner),
            Value::i8(inner) => OrdEqValue::i8(*inner),
            Value::i16(inner) => OrdEqValue::i16(*inner),
            Value::i32(inner) => OrdEqValue::i32(*inner),
            Value::i64(inner) => OrdEqValue::i64(*inner),
            Value::i128(inner) => OrdEqValue::i128(*inner),
            Value::bool(inner) => OrdEqValue::bool(*inner),
            Value::char(inner) => OrdEqValue::char(*inner),
            Value::f32(inner) => OrdEqValue::f32(OrderedFloat(*inner)),
            Value::f64(inner) => OrdEqValue::f64(OrderedFloat(*inner)),
            Value::String(inner) => OrdEqValue::String(inner),
            Value::StructValue(inner) => OrdEqValue::StructValue(inner),
            Value::EnumValue(inner) => OrdEqValue::EnumValue(inner),
            Value::TupleStructValue(inner) => OrdEqValue::TupleStructValue(inner),
            Value::TupleValue(inner) => OrdEqValue::TupleValue(inner),
            Value::List(inner) => OrdEqValue::List(inner),
            Value::Map(inner) => OrdEqValue::Map(inner),
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        OrdEqValue::from(self) == OrdEqValue::from(other)
    }
}

impl Eq for Value {}

impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Value {
    fn cmp(&self, other: &Self) -> Ordering {
        OrdEqValue::from(self).cmp(&OrdEqValue::from(other))
    }
}

macro_rules! for_each_variant {
    ($self:expr, $inner:ident => $expr:expr) => {
        match $self {
            Value::usize($inner) => $expr,
            Value::u8($inner) => $expr,
            Value::u16($inner) => $expr,
            Value::u32($inner) => $expr,
            Value::u64($inner) => $expr,
            Value::u128($inner) => $expr,
            Value::i8($inner) => $expr,
            Value::i16($inner) => $expr,
            Value::i32($inner) => $expr,
            Value::i64($inner) => $expr,
            Value::i128($inner) => $expr,
            Value::bool($inner) => $expr,
            Value::char($inner) => $expr,
            Value::f32($inner) => $expr,
            Value::f64($inner) => $expr,
            Value::String($inner) => $expr,
            Value::StructValue($inner) => $expr,
            Value::TupleStructValue($inner) => $expr,
            Value::EnumValue($inner) => $expr,
            Value::TupleValue($inner) => $expr,
            Value::List($inner) => $expr,
            Value::Map($inner) => $expr,
        }
    };
}

impl Reflect for Value {
    fn type_info(&self) -> TypeRoot {
        impl Typed for Value {
            fn build(graph: &mut TypeGraph) -> NodeId {
                graph.get_or_build_node_with::<Self, _>(|graph| {
                    OpaqueNode::new::<Self>(Default::default(), graph)
                })
            }
        }

        <Self as Typed>::type_info()
    }

    fn into_any(self: Box<Self>) -> Box<dyn Any> {
        for_each_variant!(*self, inner => Box::new(inner))
    }

    fn as_any(&self) -> &dyn Any {
        for_each_variant!(self, inner => inner)
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        for_each_variant!(self, inner => inner)
    }

    fn into_reflect(self: Box<Self>) -> Box<dyn Reflect> {
        for_each_variant!(*self, inner => Box::new(inner))
    }

    fn as_reflect(&self) -> &dyn Reflect {
        for_each_variant!(self, inner => inner)
    }

    fn as_reflect_mut(&mut self) -> &mut dyn Reflect {
        for_each_variant!(self, inner => inner)
    }

    fn reflect_owned(self: Box<Self>) -> ReflectOwned {
        match *self {
            Value::usize(inner) => ReflectOwned::Scalar(ScalarOwned::from(inner)),
            Value::u8(inner) => ReflectOwned::Scalar(ScalarOwned::from(inner)),
            Value::u16(inner) => ReflectOwned::Scalar(ScalarOwned::from(inner)),
            Value::u32(inner) => ReflectOwned::Scalar(ScalarOwned::from(inner)),
            Value::u64(inner) => ReflectOwned::Scalar(ScalarOwned::from(inner)),
            Value::u128(inner) => ReflectOwned::Scalar(ScalarOwned::from(inner)),
            Value::i8(inner) => ReflectOwned::Scalar(ScalarOwned::from(inner)),
            Value::i16(inner) => ReflectOwned::Scalar(ScalarOwned::from(inner)),
            Value::i32(inner) => ReflectOwned::Scalar(ScalarOwned::from(inner)),
            Value::i64(inner) => ReflectOwned::Scalar(ScalarOwned::from(inner)),
            Value::i128(inner) => ReflectOwned::Scalar(ScalarOwned::from(inner)),
            Value::bool(inner) => ReflectOwned::Scalar(ScalarOwned::from(inner)),
            Value::char(inner) => ReflectOwned::Scalar(ScalarOwned::from(inner)),
            Value::f32(inner) => ReflectOwned::Scalar(ScalarOwned::from(inner)),
            Value::f64(inner) => ReflectOwned::Scalar(ScalarOwned::from(inner)),
            Value::String(inner) => ReflectOwned::Scalar(ScalarOwned::from(inner)),
            Value::StructValue(inner) => ReflectOwned::Struct(inner),
            Value::EnumValue(inner) => ReflectOwned::Enum(inner),
            Value::TupleStructValue(inner) => ReflectOwned::TupleStruct(Box::new(inner)),
            Value::TupleValue(inner) => ReflectOwned::Tuple(Box::new(inner)),
            Value::List(inner) => ReflectOwned::List(Box::new(inner)),
            Value::Map(inner) => ReflectOwned::Map(Box::new(inner)),
        }
    }

    fn reflect_ref(&self) -> ReflectRef<'_> {
        match self {
            Value::usize(inner) => ReflectRef::Scalar(ScalarRef::from(*inner)),
            Value::u8(inner) => ReflectRef::Scalar(ScalarRef::from(*inner)),
            Value::u16(inner) => ReflectRef::Scalar(ScalarRef::from(*inner)),
            Value::u32(inner) => ReflectRef::Scalar(ScalarRef::from(*inner)),
            Value::u64(inner) => ReflectRef::Scalar(ScalarRef::from(*inner)),
            Value::u128(inner) => ReflectRef::Scalar(ScalarRef::from(*inner)),
            Value::i8(inner) => ReflectRef::Scalar(ScalarRef::from(*inner)),
            Value::i16(inner) => ReflectRef::Scalar(ScalarRef::from(*inner)),
            Value::i32(inner) => ReflectRef::Scalar(ScalarRef::from(*inner)),
            Value::i64(inner) => ReflectRef::Scalar(ScalarRef::from(*inner)),
            Value::i128(inner) => ReflectRef::Scalar(ScalarRef::from(*inner)),
            Value::bool(inner) => ReflectRef::Scalar(ScalarRef::from(*inner)),
            Value::char(inner) => ReflectRef::Scalar(ScalarRef::from(*inner)),
            Value::f32(inner) => ReflectRef::Scalar(ScalarRef::from(*inner)),
            Value::f64(inner) => ReflectRef::Scalar(ScalarRef::from(*inner)),
            Value::String(inner) => ReflectRef::Scalar(ScalarRef::from(inner)),
            Value::StructValue(inner) => ReflectRef::Struct(&**inner),
            Value::EnumValue(inner) => ReflectRef::Enum(&**inner),
            Value::TupleStructValue(inner) => ReflectRef::TupleStruct(inner),
            Value::TupleValue(inner) => ReflectRef::Tuple(inner),
            Value::List(inner) => ReflectRef::List(inner),
            Value::Map(inner) => ReflectRef::Map(inner),
        }
    }

    fn reflect_mut(&mut self) -> ReflectMut<'_> {
        match self {
            Value::usize(inner) => ReflectMut::Scalar(ScalarMut::from(inner)),
            Value::u8(inner) => ReflectMut::Scalar(ScalarMut::from(inner)),
            Value::u16(inner) => ReflectMut::Scalar(ScalarMut::from(inner)),
            Value::u32(inner) => ReflectMut::Scalar(ScalarMut::from(inner)),
            Value::u64(inner) => ReflectMut::Scalar(ScalarMut::from(inner)),
            Value::u128(inner) => ReflectMut::Scalar(ScalarMut::from(inner)),
            Value::i8(inner) => ReflectMut::Scalar(ScalarMut::from(inner)),
            Value::i16(inner) => ReflectMut::Scalar(ScalarMut::from(inner)),
            Value::i32(inner) => ReflectMut::Scalar(ScalarMut::from(inner)),
            Value::i64(inner) => ReflectMut::Scalar(ScalarMut::from(inner)),
            Value::i128(inner) => ReflectMut::Scalar(ScalarMut::from(inner)),
            Value::bool(inner) => ReflectMut::Scalar(ScalarMut::from(inner)),
            Value::char(inner) => ReflectMut::Scalar(ScalarMut::from(inner)),
            Value::f32(inner) => ReflectMut::Scalar(ScalarMut::from(inner)),
            Value::f64(inner) => ReflectMut::Scalar(ScalarMut::from(inner)),
            Value::String(inner) => ReflectMut::Scalar(ScalarMut::from(inner)),
            Value::StructValue(inner) => ReflectMut::Struct(&mut **inner),
            Value::EnumValue(inner) => ReflectMut::Enum(&mut **inner),
            Value::TupleStructValue(inner) => ReflectMut::TupleStruct(inner),
            Value::TupleValue(inner) => ReflectMut::Tuple(inner),
            Value::List(inner) => ReflectMut::List(inner),
            Value::Map(inner) => ReflectMut::Map(inner),
        }
    }

    fn patch(&mut self, value: &dyn Reflect) {
        for_each_variant!(self, inner => inner.patch(value))
    }

    fn to_value(&self) -> Value {
        self.clone()
    }

    fn clone_reflect(&self) -> Box<dyn Reflect> {
        Box::new(self.clone())
    }

    fn debug(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if f.alternate() {
            write!(f, "{:#?}", self)
        } else {
            write!(f, "{:?}", self)
        }
    }
}

macro_rules! from_impls {
    (
        $($ident:ident)*
    ) => {
        $(
            impl From<$ident> for Value {
                fn from(value: $ident) -> Self {
                    Value::$ident(value)
                }
            }
        )*
    };
}

impl From<StructValue> for Value {
    fn from(value: StructValue) -> Self {
        Value::StructValue(Box::new(value))
    }
}

impl From<EnumValue> for Value {
    fn from(value: EnumValue) -> Self {
        Value::EnumValue(Box::new(value))
    }
}

impl From<&str> for Value {
    fn from(value: &str) -> Self {
        value.to_owned().into()
    }
}

from_impls! {
    usize u8 u16 u32 u64 u128
    i8 i16 i32 i64 i128
    f32 f64
    bool char String
    TupleValue TupleStructValue
}
