use crate::FromReflect;
use crate::Reflect;
use crate::ReflectMut;
use crate::ReflectRef;
use crate::Tuple;
use crate::TupleValue;
use crate::Value;
use crate::ValueIter;
use crate::ValueIterMut;
use serde::Deserialize;
use serde::Serialize;
use speedy::Readable;
use speedy::Writable;
use std::any::Any;
use std::fmt;

pub trait TupleStruct: Reflect {
    fn element(&self, index: usize) -> Option<&dyn Reflect>;
    fn element_mut(&mut self, index: usize) -> Option<&mut dyn Reflect>;

    fn elements(&self) -> ValueIter<'_>;
    fn elements_mut(&mut self) -> ValueIterMut<'_>;
}

impl fmt::Debug for dyn TupleStruct {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_reflect().debug(f)
    }
}

#[derive(Default, Readable, Writable, Serialize, Deserialize, Debug, Clone)]
pub struct TupleStructValue {
    tuple: TupleValue,
}

impl TupleStructValue {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_element(self, value: impl Into<Value>) -> Self {
        Self {
            tuple: self.tuple.with_element(value),
        }
    }

    pub fn push_element(&mut self, value: impl Into<Value>) {
        self.tuple.push_element(value);
    }
}

impl Reflect for TupleStructValue {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn as_reflect(&self) -> &dyn Reflect {
        self
    }

    fn as_reflect_mut(&mut self) -> &mut dyn Reflect {
        self
    }

    fn patch(&mut self, value: &dyn Reflect) {
        if let Some(tuple) = value.reflect_ref().as_tuple_struct() {
            for (index, value) in self.elements_mut().enumerate() {
                if let Some(new_value) = tuple.element(index) {
                    value.patch(new_value);
                }
            }
        }
    }

    fn to_value(&self) -> Value {
        self.clone().into()
    }

    fn clone_reflect(&self) -> Box<dyn Reflect> {
        Box::new(self.clone())
    }

    fn debug(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if f.alternate() {
            write!(f, "{:#?}", self)
        } else {
            write!(f, "{:?}", self)
        }
    }

    fn reflect_ref(&self) -> ReflectRef<'_> {
        ReflectRef::TupleStruct(self)
    }

    fn reflect_mut(&mut self) -> ReflectMut<'_> {
        ReflectMut::TupleStruct(self)
    }
}

impl TupleStruct for TupleStructValue {
    fn element(&self, index: usize) -> Option<&dyn Reflect> {
        self.tuple.element(index)
    }

    fn element_mut(&mut self, index: usize) -> Option<&mut dyn Reflect> {
        self.tuple.element_mut(index)
    }

    fn elements(&self) -> ValueIter<'_> {
        self.tuple.elements()
    }

    fn elements_mut(&mut self) -> ValueIterMut<'_> {
        self.tuple.elements_mut()
    }
}

impl FromReflect for TupleStructValue {
    fn from_reflect(reflect: &dyn Reflect) -> Option<Self> {
        let tuple_struct = reflect.reflect_ref().as_tuple_struct()?;
        let this = tuple_struct
            .elements()
            .fold(TupleStructValue::default(), |builder, value| {
                builder.with_element(value.to_value())
            });
        Some(this)
    }
}

impl<V> FromIterator<V> for TupleStructValue
where
    V: Reflect,
{
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = V>,
    {
        let mut out = Self::default();
        for value in iter {
            out.push_element(value.to_value());
        }
        out
    }
}