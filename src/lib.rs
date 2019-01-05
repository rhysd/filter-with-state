pub struct FilterWith<I, P, S> {
    iter: I,
    predicate: P,
    state: S,
}

impl<I: Iterator, P, S> Iterator for FilterWith<I, P, S>
where
    P: FnMut(&mut S, &I::Item) -> bool,
{
    type Item = I::Item;

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let (_, upper) = self.iter.size_hint();
        (0, upper) // can't know a lower bound, due to the predicate
    }

    #[inline]
    fn next(&mut self) -> Option<I::Item> {
        for x in &mut self.iter {
            if (self.predicate)(&mut self.state, &x) {
                return Some(x);
            }
        }
        None
    }
}

pub struct FilterMapWith<I, P, S> {
    iter: I,
    predicate: P,
    state: S,
}

impl<I: Iterator, B, P, S> Iterator for FilterMapWith<I, P, S>
where
    P: FnMut(&mut S, I::Item) -> Option<B>,
{
    type Item = B;

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let (_, upper) = self.iter.size_hint();
        (0, upper) // can't know a lower bound, due to the predicate
    }

    #[inline]
    fn next(&mut self) -> Option<B> {
        for x in &mut self.iter.by_ref() {
            if let Some(y) = (self.predicate)(&mut self.state, x) {
                return Some(y);
            }
        }
        None
    }
}

pub trait FilterWithExt: Iterator {
    #[inline]
    fn filter_with<S, P>(self, state: S, predicate: P) -> FilterWith<Self, P, S>
    where
        Self: Sized,
        P: FnMut(&mut S, &Self::Item) -> bool,
    {
        FilterWith {
            iter: self,
            predicate,
            state,
        }
    }

    #[inline]
    fn filter_map_with<B, S, P>(self, state: S, predicate: P) -> FilterMapWith<Self, P, S>
    where
        Self: Sized,
        P: FnMut(&mut S, Self::Item) -> Option<B>,
    {
        FilterMapWith {
            iter: self,
            predicate,
            state,
        }
    }
}

impl<I: Iterator> FilterWithExt for I {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn filter_with_predicate() {
        let v = vec![0, 1, 2, 3, 4, 5];
        let v = v
            .into_iter()
            .filter_with(Vec::new(), |s: &mut Vec<i32>, i: &i32| {
                assert_eq!(s.len() as i32, *i);
                if s.contains(i) {
                    false
                } else {
                    s.push(*i);
                    true
                }
            })
            .collect::<Vec<_>>();
        assert_eq!(v, vec![0, 1, 2, 3, 4, 5]);
    }

    #[test]
    fn filter_with_does_not_take_ownership() {
        let v = vec![0, 1, 3, 2, 8, 14];
        let v2 = v
            .iter()
            .filter_with(0, |sum, i| {
                let b = *sum < **i;
                *sum = *sum + **i;
                b
            })
            .collect::<Vec<_>>();
        assert_eq!(v2, vec![&1, &3, &8]);

        let v3 = v
            .iter()
            .filter_with(0, |sum, i| {
                let b = *sum <= **i; // <=, not <
                *sum = *sum + **i;
                b
            })
            .collect::<Vec<_>>();
        assert_eq!(v3, vec![&0, &1, &3, &8, &14]);
    }

    #[test]
    fn unique_by_filter_with() {
        let v = vec![1, 2, 3, 2, 1, 4, 3];
        let v = v
            .into_iter()
            .filter_with(HashSet::new(), |s, i| {
                if s.contains(i) {
                    false
                } else {
                    s.insert(*i);
                    true
                }
            })
            .collect::<Vec<_>>();
        assert_eq!(v, vec![1, 2, 3, 4]);
    }

    #[test]
    fn filter_map_with_predicate() {
        let v = vec![0, 1, 2, 3, 4, 5];
        let v = v
            .into_iter()
            .filter_map_with(Vec::new(), |s: &mut Vec<i32>, i: i32| {
                assert_eq!(s.len() as i32, i);
                if s.contains(&i) {
                    None
                } else {
                    s.push(i);
                    Some(i * i)
                }
            })
            .collect::<Vec<_>>();
        assert_eq!(v, vec![0, 1, 4, 9, 16, 25]);
    }

    #[test]
    fn unique_map_by_filter_with() {
        let v = vec![1, 2, 3, 2, 1, 4, 3];
        let v = v
            .into_iter()
            .filter_map_with(HashSet::new(), |s, i| {
                if s.contains(&i) {
                    None
                } else {
                    s.insert(i);
                    Some((i * i).to_string())
                }
            })
            .collect::<Vec<_>>();
        assert_eq!(
            v,
            vec![
                "1".to_string(),
                "4".to_string(),
                "9".to_string(),
                "16".to_string()
            ]
        );
    }

}
