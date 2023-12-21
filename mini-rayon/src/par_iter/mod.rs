mod len;
mod map;
mod slice;
mod reduce;
mod collect;

pub use self::map::Map;
pub use self::len::ParallelLen;

pub trait IntoParallelIterator {
    type Iter: ParallelIterator<Item=Self::Item>;
    type Item;

    fn into_par_iter(self) -> Self::Iter;
}

pub trait ParallelIteratorState: Sized {
    type Item;
    type Shared: Sync;

    fn len(&mut self) -> ParallelLen;

    fn split_at(self, index: usize) -> (Self, Self);

    fn for_each<OP>(self, shared: &Self::Shared, op: OP)
    where
        OP: FnMut(Self::Item);
}

pub trait ParallelIterator {
    type Item;
    type Shared: Sync;
    type State: ParallelIteratorState<Shared=Self::Shared, Item=Self::Item> + Send;

    fn map<MapOp, R>(self, map_op: MapOp) -> Map<Self, MapOp>
    where
        MapOp: Fn(Self::Item) -> R,
        Self: Sized,
    {
        Map::new(self, map_op)
    }
}