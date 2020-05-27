use crate::HashMap;
use rayon::iter::{FromParallelIterator, IntoParallelIterator, ParallelExtend, ParallelIterator};
use std::hash::{BuildHasher, Hash};

impl<K, V, S> ParallelExtend<(K, V)> for HashMap<K, V, S>
where
    K: Clone + Hash + Ord + Send + Sync + 'static,
    V: Send + Sync + 'static,
    S: BuildHasher + Sync,
{
    fn par_extend<I>(&mut self, par_iter: I)
    where
        I: IntoParallelIterator<Item = (K, V)>,
    {
        (&*self).par_extend(par_iter);
    }
}

impl<K, V, S> ParallelExtend<(K, V)> for &HashMap<K, V, S>
where
    K: Clone + Hash + Ord + Send + Sync + 'static,
    V: Send + Sync + 'static,
    S: BuildHasher + Sync,
{
    fn par_extend<I>(&mut self, par_iter: I)
    where
        I: IntoParallelIterator<Item = (K, V)>,
    {
        par_iter.into_par_iter().for_each_init(
            || self.guard(),
            |guard, (k, v)| {
                self.insert(k, v, &guard);
            },
        );
    }
}

impl<K, V, S> FromParallelIterator<(K, V)> for HashMap<K, V, S>
where
    K: Clone + Hash + Ord + Send + Sync + 'static,
    V: Send + Sync + 'static,
    S: BuildHasher + Default + Sync,
{
    fn from_par_iter<I>(par_iter: I) -> Self
    where
        I: IntoParallelIterator<Item = (K, V)>,
    {
        let mut created_map = HashMap::with_hasher(S::default());
        created_map.par_extend(par_iter);
        created_map
    }
}

#[cfg(test)]
mod test {
    use crate::HashMap;
    use rayon::iter::{FromParallelIterator, IntoParallelIterator, ParallelExtend};

    #[test]
    fn parallel_extend_by_nothing() {
        let to_extend_with = Vec::new();

        let mut map = HashMap::new();
        let guard = map.guard();
        map.insert(1, 2, &guard);
        map.insert(3, 4, &guard);

        map.par_extend(to_extend_with.into_par_iter());

        assert_eq!(map.len(), 2);

        assert_eq!(map.get(&1, &guard), Some(&2));
        assert_eq!(map.get(&3, &guard), Some(&4));
    }

    #[test]
    fn parallel_extend_by_a_bunch() {
        let mut to_extend_with = Vec::new();
        for i in 0..100 {
            to_extend_with.push((i + 100, i * 10));
        }

        let mut map = HashMap::new();
        let guard = map.guard();
        map.insert(1, 2, &guard);
        map.insert(3, 4, &guard);

        map.par_extend(to_extend_with.into_par_iter());
        assert_eq!(map.len(), 102);

        assert_eq!(map.get(&1, &guard), Some(&2));
        assert_eq!(map.get(&3, &guard), Some(&4));
        assert_eq!(map.get(&100, &guard), Some(&0));
        assert_eq!(map.get(&199, &guard), Some(&990));
    }

    #[test]
    fn from_empty_parallel_iter() {
        let to_create_from: Vec<(i32, i32)> = Vec::new();
        let created_map: HashMap<i32, i32> = HashMap::from_par_iter(to_create_from.into_par_iter());
        assert_eq!(created_map.len(), 0);
    }

    #[test]
    fn from_large_parallel_iter() {
        let mut to_create_from: Vec<(i32, i32)> = Vec::new();
        for i in 0..100 {
            to_create_from.push((i + 100, i * 10));
        }
        let created_map: HashMap<i32, i32> = HashMap::from_par_iter(to_create_from.into_par_iter());
        assert_eq!(created_map.len(), 100);

        let guard = created_map.guard();
        assert_eq!(created_map.get(&100, &guard), Some(&0));
        assert_eq!(created_map.get(&199, &guard), Some(&990));
    }
}
