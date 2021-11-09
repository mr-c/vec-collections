use crate::AbstractRadixTreeMut;
use std::{collections::BTreeMap, sync::Arc};

use super::{
    lazy::Lazy, location, offset_from, AbstractRadixTree, Fragment, RadixTree, TKey, TValue,
};
use rkyv::{
    ser::{ScratchSpace, Serializer, SharedSerializeRegistry},
    vec::ArchivedVec,
    Archive, Archived, Resolver, Serialize,
};

#[derive(Clone)]
pub struct LazyRadixTree<'a, K, V>
where
    K: TKey,
    V: TValue,
{
    pub(crate) prefix: Fragment<K>,
    pub(crate) value: Option<V>,
    /// the children are lazy loaded at the time of first access.
    pub(crate) children: Lazy<&'a [Archived<LazyRadixTree<'a, K, V>>], Arc<Vec<Self>>>,
}

impl<'a, K: TKey, V: TValue> Default for LazyRadixTree<'a, K, V> {
    fn default() -> Self {
        Self {
            prefix: Default::default(),
            value: Default::default(),
            children: Default::default(),
        }
    }
}

impl<'a, K: TKey, V: TValue> AbstractRadixTree<K, V> for LazyRadixTree<'a, K, V> {
    type Materialized = LazyRadixTree<'a, K, V>;

    fn prefix(&self) -> &[K] {
        &self.prefix
    }

    fn value(&self) -> Option<&V> {
        self.value.as_ref()
    }

    fn children(&self) -> &[Self] {
        self.children_arc().as_ref()
    }
}

impl<'a, K: TKey, V: TValue> AbstractRadixTreeMut<K, V> for LazyRadixTree<'a, K, V> {
    fn new(prefix: Fragment<K>, value: Option<V>, children: Vec<Self>) -> Self {
        let children = Lazy::initialized(Arc::new(children));
        Self {
            prefix,
            value,
            children,
        }
    }

    fn value_mut(&mut self) -> &mut Option<V> {
        &mut self.value
    }

    fn prefix_mut(&mut self) -> &mut Fragment<K> {
        &mut self.prefix
    }

    fn children_mut(&mut self) -> &mut Vec<Self> {
        Arc::make_mut(self.children_arc_mut())
    }
}

impl<K: TKey, V: TValue> From<RadixTree<K, V>> for LazyRadixTree<'static, K, V> {
    fn from(value: RadixTree<K, V>) -> Self {
        let RadixTree {
            prefix,
            value,
            children,
        } = value;
        let children = children.into_iter().map(Self::from).collect::<Vec<_>>();
        let children = Lazy::initialized(Arc::new(children));
        Self {
            prefix,
            value,
            children,
        }
    }
}

impl<'a, K: TKey, V: TValue> LazyRadixTree<'a, K, V> {
    pub(crate) fn children_arc(&self) -> &Arc<Vec<Self>> {
        self.children.get_or_create(materialize_shallow)
    }

    pub(crate) fn children_arc_mut(&mut self) -> &mut Arc<Vec<Self>> {
        self.children.get_or_create_mut(materialize_shallow)
    }

    fn maybe_arc(&self) -> Option<&Arc<Vec<Self>>> {
        self.children.get()
    }

    pub fn all_arcs(&self, into: &mut BTreeMap<usize, Arc<Vec<Self>>>) {
        if let Some(children) = self.maybe_arc() {
            into.insert(location(children.as_ref()), children.clone());
            for child in children.iter() {
                child.all_arcs(into);
            }
        }
    }
}

impl<'a, K: TKey + Archive<Archived = K>, V: TValue + Archive<Archived = V>>
    From<&'a ArchivedLazyRadixTree<K, V>> for LazyRadixTree<'a, K, V>
{
    fn from(value: &'a ArchivedLazyRadixTree<K, V>) -> Self {
        let children = value.children().iter().map(Self::from).collect::<Vec<_>>();
        let children = Lazy::initialized(Arc::new(children));
        LazyRadixTree {
            prefix: value.prefix().into(),
            value: value.value().cloned(),
            children,
        }
    }
}

impl<K: TKey, V: TValue> AbstractRadixTree<K, V> for ArchivedLazyRadixTree<K, V> {
    type Materialized = LazyRadixTree<'static, K, V>;

    fn prefix(&self) -> &[K] {
        &self.prefix
    }

    fn value(&self) -> Option<&V> {
        self.value.as_ref()
    }

    fn children(&self) -> &[Self] {
        &self.children
    }
}

fn materialize_shallow<K: TKey, V: TValue>(
    children: &[ArchivedLazyRadixTree<K, V>],
) -> Arc<Vec<LazyRadixTree<K, V>>> {
    Arc::new(
        children
            .iter()
            .map(|child| LazyRadixTree {
                prefix: child.prefix.as_ref().into(),
                value: child.value.as_ref().cloned(),
                children: Lazy::uninitialized(child.children.as_ref()),
            })
            .collect(),
    )
}

pub struct LazyRadixTreeResolver<K: TKey + Archive, V: TValue + Archive> {
    prefix: Resolver<Vec<K>>,
    value: Resolver<Option<V>>,
    children: Resolver<Arc<Vec<LazyRadixTree<'static, K, V>>>>,
}

#[repr(C)]
pub struct ArchivedLazyRadixTree<K, V>
where
    K: TKey,
    V: TValue,
{
    pub(crate) prefix: Archived<Vec<K>>,
    pub(crate) value: Archived<Option<V>>,
    pub(crate) children: Archived<Arc<Vec<LazyRadixTree<'static, K, V>>>>,
}

impl<'a, K: TKey, V: TValue> Archive for LazyRadixTree<'a, K, V> {
    type Archived = ArchivedLazyRadixTree<K, V>;

    type Resolver = LazyRadixTreeResolver<K, V>;

    unsafe fn resolve(&self, pos: usize, resolver: Self::Resolver, out: *mut Self::Archived) {
        let LazyRadixTreeResolver {
            prefix,
            value,
            children,
        } = resolver;
        let ptr = &mut (*out).prefix;
        ArchivedVec::resolve_from_slice(self.prefix(), pos + offset_from(out, ptr), prefix, ptr);
        let ptr = &mut (*out).value;
        self.value()
            .cloned()
            .resolve(pos + offset_from(out, ptr), value, ptr);
        let ptr = &mut (*out).children;
        self.children_arc()
            .resolve(pos + offset_from(out, ptr), children, ptr);
    }
}

impl<'a, S, K, V> Serialize<S> for LazyRadixTree<'a, K, V>
where
    K: TKey + Serialize<S>,
    V: TValue + Serialize<S>,
    S: ScratchSpace + Serializer + SharedSerializeRegistry,
{
    fn serialize(&self, serializer: &mut S) -> Result<Self::Resolver, S::Error> {
        let prefix = rkyv::vec::ArchivedVec::serialize_from_slice(self.prefix(), serializer)?;
        let value = self.value().cloned().serialize(serializer)?;
        let arc = self.children_arc();
        let arc: &Arc<Vec<LazyRadixTree<'static, K, V>>> = unsafe { std::mem::transmute(arc) };
        let children = arc.serialize(serializer)?;
        Ok(LazyRadixTreeResolver {
            prefix,
            value,
            children,
        })
    }
}
