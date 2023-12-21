use std::marker::PhantomData;
use super::{ParallelIterator, ParallelIteratorState, ParallelLen};

pub struct Map<M, MapOp> {
    base: M,
    map_op: MapOp,
}

impl<M, MapOp> Map<M, MapOp> {
    pub fn new(base: M, map_op: MapOp) -> Map<M, MapOp> {
        Map {
            base,
            map_op,
        }
    }
}

impl<M, MapOp, R> ParallelIterator for Map<M, MapOp>
where
    M: ParallelIterator,
    MapOp: Fn(M::Item) -> R + Sync + Send,
{
    type Item = R;
    type Shared = MapShared<M, MapOp>;
    type State = MapState<M, MapOp>;
}

pub struct MapShared<M, MapOp>
where
    M: ParallelIterator,
{
    base: M::Shared,
    map_op: MapOp,
}

pub struct MapState<M, MapOp>
where
    M: ParallelIterator,
{
    base: M::State,
    map_op: PhantomData<MapOp>,
}

impl<M, MapOp, R> ParallelIteratorState for MapState<M, MapOp>
where
    M: ParallelIterator,
    MapOp: Fn(M::Item) -> R + Sync,
{
    type Item = R;
    type Shared = MapShared<M, MapOp>;

    fn len(&mut self) -> ParallelLen {
        self.base.len()
    }

    fn split_at(self, index: usize) -> (Self, Self) {
        let (left, right) = self.base.split_at(index);
        (MapState { base: left, map_op: PhantomData },
         MapState { base: right, map_op: PhantomData })
    }

    fn for_each<F>(self, shared: &Self::Shared, mut op: F)
    where
        F: FnMut(R)
    {
        self.base.for_each(&shared.base, |item| {
            op((shared.map_op)(item));
        });
    }
}