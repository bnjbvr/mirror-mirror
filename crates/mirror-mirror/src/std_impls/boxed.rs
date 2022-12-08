use alloc::boxed::Box;
use core::any::Any;
use core::fmt;

use crate::reflect_debug;
use crate::type_info::graph::Id;
use crate::type_info::graph::TypeInfoGraph;
use crate::FromReflect;
use crate::Reflect;
use crate::ReflectMut;
use crate::ReflectRef;
use crate::TypeInfoRoot;
use crate::Typed;
use crate::Value;

impl<T> Reflect for Box<T>
where
    T: Reflect + Typed,
{
    fn type_info(&self) -> TypeInfoRoot {
        impl<T> Typed for Box<T>
        where
            T: Typed,
        {
            fn build(graph: &mut TypeInfoGraph) -> Id {
                T::build(graph)
            }
        }

        <T as Typed>::type_info()
    }

    fn as_any(&self) -> &dyn Any {
        <T as Reflect>::as_any(self)
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        <T as Reflect>::as_any_mut(self)
    }

    fn as_reflect(&self) -> &dyn Reflect {
        <T as Reflect>::as_reflect(self)
    }

    fn as_reflect_mut(&mut self) -> &mut dyn Reflect {
        <T as Reflect>::as_reflect_mut(self)
    }

    fn reflect_ref(&self) -> ReflectRef<'_> {
        <T as Reflect>::reflect_ref(self)
    }

    fn reflect_mut(&mut self) -> ReflectMut<'_> {
        <T as Reflect>::reflect_mut(self)
    }

    fn patch(&mut self, value: &dyn Reflect) {
        <T as Reflect>::patch(self, value)
    }

    fn to_value(&self) -> Value {
        <T as Reflect>::to_value(self)
    }

    fn clone_reflect(&self) -> Box<dyn Reflect> {
        <T as Reflect>::clone_reflect(self)
    }

    fn debug(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        reflect_debug(self, f)
    }
}

impl<T> FromReflect for Box<T>
where
    T: FromReflect + Typed,
{
    fn from_reflect(reflect: &dyn Reflect) -> Option<Self> {
        Some(Box::new(T::from_reflect(reflect)?))
    }
}

impl<T> From<Box<T>> for Value
where
    T: Into<Value>,
{
    fn from(boxed: Box<T>) -> Self {
        (*boxed).into()
    }
}