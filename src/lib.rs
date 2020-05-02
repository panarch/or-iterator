use core::cmp::max;

pub trait OrIterator: Sized {
    fn or<I2: Iterator>(self, other: I2) -> Or<Self, I2>;
}

impl<I1: Iterator> OrIterator for I1 {
    fn or<I2: Iterator>(self: I1, other: I2) -> Or<I1, I2> {
        Or {
            iter1: self,
            iter2: other,
            state: Initial,
        }
    }
}

enum State {
    Initial,
    InIter1,
    InIter2,
}

pub struct Or<I1, I2> {
    iter1: I1,
    iter2: I2,
    state: State,
}

use State::{InIter1, InIter2, Initial};

impl<I1, I2> Iterator for Or<I1, I2>
where
    I1: Iterator,
    I2: Iterator<Item = I1::Item>,
{
    type Item = I1::Item;

    fn next(&mut self) -> Option<Self::Item> {
        match self.state {
            Initial => match self.iter1.next() {
                x @ Some(_) => {
                    self.state = InIter1;
                    x
                }
                None => {
                    self.state = InIter2;
                    self.iter2.next()
                }
            },
            InIter1 => self.iter1.next(),
            InIter2 => self.iter2.next(),
        }
    }

    // implementations below this for performance reasons
    fn size_hint(&self) -> (usize, Option<usize>) {
        match self.state {
            Initial => match self.iter1.size_hint() {
                (_, Some(0)) => self.iter2.size_hint(),
                (0, i1_high) => {
                    let (i2_low, i2_high) = self.iter2.size_hint();
                    let low = if i2_low > 0 { 1 } else { 0 };
                    let high = i1_high.and_then(|h1| i2_high.map(|h2| max(h1, h2)));
                    (low, high)
                }
                i1_hint => i1_hint,
            },
            InIter1 => self.iter1.size_hint(),
            InIter2 => self.iter2.size_hint(),
        }
    }

    fn fold<B, F: FnMut(B, Self::Item) -> B>(mut self, init: B, mut f: F) -> B {
        match self.state {
            Initial => match self.iter1.next() {
                Some(x) => self.iter1.fold(f(init, x), f),
                None => self.iter2.fold(init, f),
            },
            InIter1 => self.iter1.fold(init, f),
            InIter2 => self.iter2.fold(init, f),
        }
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        match self.state {
            Initial => match self.iter1.next() {
                x @ Some(_) => {
                    self.state = InIter1;
                    if n == 0 {
                        x
                    } else {
                        self.iter1.nth(n - 1)
                    }
                }
                None => {
                    self.state = InIter1;
                    self.iter2.nth(n)
                }
            },
            InIter1 => self.iter1.nth(n),
            InIter2 => self.iter2.nth(n),
        }
    }

    fn last(mut self) -> Option<Self::Item> {
        match self.state {
            Initial => match self.iter1.next() {
                x @ Some(_) => self.iter1.last().or(x),
                None => self.iter2.last(),
            },
            InIter1 => self.iter1.last(),
            InIter2 => self.iter2.last(),
        }
    }

    fn count(mut self) -> usize {
        match self.state {
            Initial => match self.iter1.next() {
                Some(_) => 1 + self.iter1.count(),
                None => self.iter2.count(),
            },
            InIter1 => self.iter1.count(),
            InIter2 => self.iter2.count(),
        }
    }
}

impl<I1, I2> ExactSizeIterator for Or<I1, I2>
where
    I1: ExactSizeIterator,
    I2: ExactSizeIterator<Item = I1::Item>,
{
}

#[cfg(test)]
mod tests {
    use super::OrIterator;
    use std::iter::{empty, once};

    #[test]
    fn basic() {
        let i1 = once(1);
        let i2 = once(3);
        let v = i1.or(i2).collect::<Vec<i32>>();
        assert_eq!(vec![1], v);

        let i1 = empty();
        let i2 = once(3);
        let v = i1.or(i2).collect::<Vec<i32>>();
        assert_eq!(vec![3], v);
    }

    #[test]
    fn nth() {
        let v1 = vec![1, 2, 3];
        let v2 = vec![4, 5];
        let mut or = v1.into_iter().or(v2.into_iter());
        assert_eq!(Some(1), or.nth(0));
        assert_eq!(Some(3), or.nth(1));
        assert_eq!(None, or.nth(3));

        let v1 = vec![];
        let v2 = vec![4, 5];
        let mut or = v1.into_iter().or(v2.into_iter());
        assert_eq!(Some(5), or.nth(1));
        assert_eq!(None, or.nth(0));
    }

    #[test]
    fn len() {
        let v1 = vec![1, 2, 3];
        let v2 = vec![4, 5];
        let or = v1.iter().or(v2.iter());
        assert_eq!(3, or.len());

        let v1 = vec![];
        let v2 = vec![4, 5];
        let or = v1.iter().or(v2.iter());
        assert_eq!(2, or.len());
    }

    #[test]
    fn count() {
        let v1 = vec![1, 2, 3];
        let v2 = vec![4, 5];
        let or = v1.iter().or(v2.iter());
        assert_eq!(3, or.count());

        let v1 = vec![];
        let v2 = vec![4, 5];
        let or = v1.iter().or(v2.iter());
        assert_eq!(2, or.count());
    }

    #[test]
    fn last() {
        let v1 = vec![1, 2, 3];
        let v2 = vec![4, 5];
        let or = v1.into_iter().or(v2.into_iter());
        assert_eq!(Some(3), or.last());

        let v1 = vec![];
        let v2 = vec![4, 5];
        let or = v1.into_iter().or(v2.into_iter());
        assert_eq!(Some(5), or.last());
    }

    #[test]
    fn fold() {
        let v1 = vec![1, 2, 3];
        let v2 = vec![4, 5];
        let or = v1.iter().or(v2.iter());
        let sum = or.fold(0, |n, v| n + v);
        assert_eq!(6, sum);

        let v1 = vec![];
        let v2 = vec![4, 5];
        let or = v1.iter().or(v2.iter());
        let sum = or.fold(0, |n, v| n + v);
        assert_eq!(9, sum);
    }
}
