use alloc::borrow::ToOwned;
use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use core::any::type_name;
use core::any::TypeId;
use core::ops::Deref;

use super::*;
use crate::Value;

#[derive(Clone, Copy, Hash, PartialEq, PartialOrd, Ord, Eq, Debug)]
#[cfg_attr(feature = "speedy", derive(speedy::Readable, speedy::Writable))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct NodeId(u64);

impl NodeId {
    fn new<T>() -> Self
    where
        T: 'static,
    {
        use core::hash::Hash;
        use core::hash::Hasher;

        let mut hasher = ahash::AHasher::default();
        TypeId::of::<T>().hash(&mut hasher);
        Self(hasher.finish())
    }
}

#[derive(Debug, Copy, Clone)]
pub(super) struct WithId<T> {
    pub(super) id: NodeId,
    inner: T,
}

impl<T> WithId<T> {
    pub(super) fn new(id: NodeId, inner: T) -> Self {
        Self { id, inner }
    }
}

impl<T> Deref for WithId<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

#[derive(Default, Debug, Clone)]
#[cfg_attr(feature = "speedy", derive(speedy::Readable, speedy::Writable))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TypeGraph {
    pub(super) map: BTreeMap<NodeId, Option<TypeNode>>,
}

impl TypeGraph {
    pub(super) fn get(&self, id: NodeId) -> &TypeNode {
        const ERROR: &str = "no node found in graph. This is a bug. Please open an issue.";
        self.map.get(&id).expect(ERROR).as_ref().expect(ERROR)
    }

    pub fn get_or_build_node_with<T, I>(&mut self, f: impl FnOnce(&mut Self) -> I) -> NodeId
    where
        I: Into<TypeNode>,
        T: Typed,
    {
        let id = NodeId::new::<T>();
        match self.map.get(&id) {
            // the data is already there
            Some(Some(_)) => id,
            // someone else is currently inserting the data
            Some(None) => id,
            // the data isn't there yet
            None => {
                self.map.insert(id, None);
                let info = f(self).into();
                self.map.insert(id, Some(info));
                id
            }
        }
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "speedy", derive(speedy::Readable, speedy::Writable))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum TypeNode {
    Struct(StructNode),
    TupleStruct(TupleStructNode),
    Tuple(TupleNode),
    Enum(EnumNode),
    List(ListNode),
    Array(ArrayNode),
    Map(MapNode),
    Scalar(ScalarNode),
    Opaque(OpaqueNode),
}

macro_rules! impl_from {
    ($variant:ident($inner:ident)) => {
        impl From<$inner> for TypeNode {
            fn from(inner: $inner) -> Self {
                Self::$variant(inner)
            }
        }
    };
}

impl_from! { Struct(StructNode) }
impl_from! { TupleStruct(TupleStructNode) }
impl_from! { Tuple(TupleNode) }
impl_from! { Enum(EnumNode) }
impl_from! { List(ListNode) }
impl_from! { Array(ArrayNode) }
impl_from! { Map(MapNode) }
impl_from! { Scalar(ScalarNode) }
impl_from! { Opaque(OpaqueNode) }

#[derive(Debug, Clone)]
#[cfg_attr(feature = "speedy", derive(speedy::Readable, speedy::Writable))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct StructNode {
    pub(super) type_name: String,
    pub(super) fields: BTreeMap<String, NamedFieldNode>,
    pub(super) field_names: Box<[String]>,
    pub(super) metadata: BTreeMap<String, Value>,
    pub(super) docs: Box<[String]>,
}

impl StructNode {
    pub fn new<T>(
        fields: &[NamedFieldNode],
        metadata: BTreeMap<&'static str, Value>,
        docs: &[&'static str],
    ) -> Self
    where
        T: Typed,
    {
        Self {
            type_name: type_name::<T>().to_owned(),
            fields: fields
                .iter()
                .map(|field| (field.name.clone(), field.clone()))
                .collect(),
            field_names: fields.iter().map(|field| field.name.clone()).collect(),
            metadata: map_metadata(metadata),
            docs: map_docs(docs),
        }
    }
}

fn map_metadata(metadata: BTreeMap<&'static str, Value>) -> BTreeMap<String, Value> {
    metadata
        .into_iter()
        .map(|(key, value)| (key.to_owned(), value))
        .collect()
}

fn map_docs(docs: &[&'static str]) -> Box<[String]> {
    docs.iter().map(|s| (*s).to_owned()).collect()
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "speedy", derive(speedy::Readable, speedy::Writable))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TupleStructNode {
    pub(super) type_name: String,
    pub(super) fields: Vec<UnnamedFieldNode>,
    pub(super) metadata: BTreeMap<String, Value>,
    pub(super) docs: Box<[String]>,
}

impl TupleStructNode {
    pub fn new<T>(
        fields: &[UnnamedFieldNode],
        metadata: BTreeMap<&'static str, Value>,
        docs: &[&'static str],
    ) -> Self
    where
        T: Typed,
    {
        Self {
            type_name: type_name::<T>().to_owned(),
            fields: fields.to_vec(),
            metadata: map_metadata(metadata),
            docs: map_docs(docs),
        }
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "speedy", derive(speedy::Readable, speedy::Writable))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct EnumNode {
    pub(super) type_name: String,
    pub(super) variants: Vec<VariantNode>,
    pub(super) metadata: BTreeMap<String, Value>,
    pub(super) docs: Box<[String]>,
}

impl EnumNode {
    pub fn new<T>(
        variants: &[VariantNode],
        metadata: BTreeMap<&'static str, Value>,
        docs: &[&'static str],
    ) -> Self
    where
        T: Typed,
    {
        Self {
            type_name: type_name::<T>().to_owned(),
            variants: variants.to_vec(),
            metadata: map_metadata(metadata),
            docs: map_docs(docs),
        }
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "speedy", derive(speedy::Readable, speedy::Writable))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum VariantNode {
    Struct(StructVariantNode),
    Tuple(TupleVariantNode),
    Unit(UnitVariantNode),
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "speedy", derive(speedy::Readable, speedy::Writable))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct StructVariantNode {
    pub(super) name: String,
    pub(super) fields: BTreeMap<String, NamedFieldNode>,
    pub(super) field_names: Box<[String]>,
    pub(super) metadata: BTreeMap<String, Value>,
    pub(super) docs: Box<[String]>,
}

impl StructVariantNode {
    pub fn new(
        name: &'static str,
        fields: &[NamedFieldNode],
        metadata: BTreeMap<&'static str, Value>,
        docs: &[&'static str],
    ) -> Self {
        Self {
            name: name.to_owned(),
            fields: fields
                .iter()
                .map(|field| (field.name.clone(), field.clone()))
                .collect(),
            field_names: fields.iter().map(|field| field.name.clone()).collect(),
            metadata: map_metadata(metadata),
            docs: map_docs(docs),
        }
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "speedy", derive(speedy::Readable, speedy::Writable))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TupleVariantNode {
    pub(super) name: String,
    pub(super) fields: Vec<UnnamedFieldNode>,
    pub(super) metadata: BTreeMap<String, Value>,
    pub(super) docs: Box<[String]>,
}

impl TupleVariantNode {
    pub fn new(
        name: &'static str,
        fields: &[UnnamedFieldNode],
        metadata: BTreeMap<&'static str, Value>,
        docs: &[&'static str],
    ) -> Self {
        Self {
            name: name.to_owned(),
            fields: fields.to_vec(),
            metadata: map_metadata(metadata),
            docs: map_docs(docs),
        }
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "speedy", derive(speedy::Readable, speedy::Writable))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct UnitVariantNode {
    pub(super) name: String,
    pub(super) metadata: BTreeMap<String, Value>,
    pub(super) docs: Box<[String]>,
}

impl UnitVariantNode {
    pub fn new(
        name: &'static str,
        metadata: BTreeMap<&'static str, Value>,
        docs: &[&'static str],
    ) -> Self {
        Self {
            name: name.to_owned(),
            metadata: map_metadata(metadata),
            docs: map_docs(docs),
        }
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "speedy", derive(speedy::Readable, speedy::Writable))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TupleNode {
    pub(super) type_name: String,
    pub(super) fields: Vec<UnnamedFieldNode>,
    pub(super) metadata: BTreeMap<String, Value>,
    pub(super) docs: Box<[String]>,
}

impl TupleNode {
    pub fn new<T>(
        fields: &[UnnamedFieldNode],
        metadata: BTreeMap<&'static str, Value>,
        docs: &[&'static str],
    ) -> Self
    where
        T: Typed,
    {
        Self {
            type_name: type_name::<T>().to_owned(),
            fields: fields.to_vec(),
            metadata: map_metadata(metadata),
            docs: map_docs(docs),
        }
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "speedy", derive(speedy::Readable, speedy::Writable))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct NamedFieldNode {
    pub(super) name: String,
    pub(super) id: NodeId,
    pub(super) metadata: BTreeMap<String, Value>,
    pub(super) docs: Box<[String]>,
}

impl NamedFieldNode {
    pub fn new<T>(
        name: &'static str,
        metadata: BTreeMap<&'static str, Value>,
        docs: &[&'static str],
        graph: &mut TypeGraph,
    ) -> Self
    where
        T: Typed,
    {
        Self {
            name: name.to_owned(),
            id: T::build(graph),
            metadata: map_metadata(metadata),
            docs: map_docs(docs),
        }
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "speedy", derive(speedy::Readable, speedy::Writable))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct UnnamedFieldNode {
    pub(super) id: NodeId,
    pub(super) metadata: BTreeMap<String, Value>,
    pub(super) docs: Box<[String]>,
}

impl UnnamedFieldNode {
    pub fn new<T>(
        metadata: BTreeMap<&'static str, Value>,
        docs: &[&'static str],
        graph: &mut TypeGraph,
    ) -> Self
    where
        T: Typed,
    {
        Self {
            id: T::build(graph),
            metadata: map_metadata(metadata),
            docs: map_docs(docs),
        }
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "speedy", derive(speedy::Readable, speedy::Writable))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ArrayNode {
    pub(super) type_name: String,
    pub(super) field_type_id: NodeId,
    pub(super) len: usize,
}

impl ArrayNode {
    pub(crate) fn new<L, T, const N: usize>(graph: &mut TypeGraph) -> Self
    where
        L: Typed,
        T: Typed,
    {
        Self {
            type_name: type_name::<L>().to_owned(),
            field_type_id: T::build(graph),
            len: N,
        }
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "speedy", derive(speedy::Readable, speedy::Writable))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ListNode {
    pub(super) type_name: String,
    pub(super) field_type_id: NodeId,
}

impl ListNode {
    pub(crate) fn new<L, T>(graph: &mut TypeGraph) -> Self
    where
        L: Typed,
        T: Typed,
    {
        Self {
            type_name: type_name::<L>().to_owned(),
            field_type_id: T::build(graph),
        }
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "speedy", derive(speedy::Readable, speedy::Writable))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MapNode {
    pub(super) type_name: String,
    pub(super) key_type_id: NodeId,
    pub(super) value_type_id: NodeId,
}

impl MapNode {
    pub(crate) fn new<M, K, V>(graph: &mut TypeGraph) -> Self
    where
        M: Typed,
        K: Typed,
        V: Typed,
    {
        Self {
            type_name: type_name::<M>().to_owned(),
            key_type_id: K::build(graph),
            value_type_id: V::build(graph),
        }
    }
}

#[derive(Debug, Clone)]
#[allow(non_camel_case_types)]
#[cfg_attr(feature = "speedy", derive(speedy::Readable, speedy::Writable))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ScalarNode {
    usize,
    u8,
    u16,
    u32,
    u64,
    u128,
    i8,
    i16,
    i32,
    i64,
    i128,
    bool,
    char,
    f32,
    f64,
    String,
}

macro_rules! scalar_typed {
    ($($ty:ident)*) => {
        $(
            impl Typed for $ty {
                fn build(graph: &mut TypeGraph) -> NodeId {
                    graph.get_or_build_node_with::<Self, _>(|_graph| ScalarNode::$ty)
                }
            }
        )*
    };
}

scalar_typed! {
    usize u8 u16 u32 u64 u128
    i8 i16 i32 i64 i128
    f32 f64
    bool char String
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "speedy", derive(speedy::Readable, speedy::Writable))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct OpaqueNode {
    pub(super) type_name: String,
    pub(super) metadata: BTreeMap<String, Value>,
}

impl OpaqueNode {
    pub fn new<T>(metadata: BTreeMap<&'static str, Value>, _graph: &mut TypeGraph) -> Self
    where
        T: Typed,
    {
        Self {
            type_name: type_name::<T>().to_owned(),
            metadata: map_metadata(metadata),
        }
    }
}
