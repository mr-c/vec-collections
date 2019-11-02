use crate::{
    BoolOpMergeState, EarlyOut, InPlaceMergeState, MergeOperation, MergeState, VecMergeState,
};
use alga::general::{JoinSemilattice, MeetSemilattice};
use std::fmt::Debug;
use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Sub, SubAssign};

struct SetUnionOp();
struct SetIntersectionOp();
struct SetXorOp();
struct SetDiffOpt();

#[derive(Clone, Hash)]
pub struct ArraySet<T>(Vec<T>);

impl<'a, T: Ord, I: MergeState<T, T>> MergeOperation<'a, T, T, I> for SetUnionOp {
    fn cmp(&self, a: &T, b: &T) -> std::cmp::Ordering {
        a.cmp(b)
    }
    fn from_a(&self, m: &mut I, n: usize) -> EarlyOut {
        // println!("{:?}", m);
        // println!("move_a {}", n);
        m.move_a(n)
        // println!("{:?}\n", m);
    }
    fn from_b(&self, m: &mut I, n: usize) -> EarlyOut {
        // println!("{:?}", m);
        // println!("move_b {}", n);
        m.move_b(n)
        // println!("{:?}\n", m);
    }
    fn collision(&self, m: &mut I) -> EarlyOut {
        // println!("{:?}", m);
        // println!("move_a 1");
        // println!("skip_b 1");
        m.move_a(1)?;
        m.skip_b(1)
        // println!("{:?}\n", m);
    }
}

impl<'a, T: Ord, I: MergeState<T, T>> MergeOperation<'a, T, T, I> for SetIntersectionOp {
    fn cmp(&self, a: &T, b: &T) -> std::cmp::Ordering {
        a.cmp(b)
    }
    fn from_a(&self, m: &mut I, n: usize) -> EarlyOut {
        // println!("{:?}", m);
        // println!("move_a {}", n);
        m.skip_a(n)
        // println!("{:?}\n", m);
    }
    fn from_b(&self, m: &mut I, n: usize) -> EarlyOut {
        // println!("{:?}", m);
        // println!("move_b {}", n);
        m.skip_b(n)
        // println!("{:?}\n", m);
    }
    fn collision(&self, m: &mut I) -> EarlyOut {
        // println!("{:?}", m);
        // println!("move_a 1");
        // println!("skip_b 1");
        m.move_a(1)?;
        m.skip_b(1)
        // println!("{:?}\n", m);
    }
}

impl<'a, T: Ord, I: MergeState<T, T>> MergeOperation<'a, T, T, I> for SetDiffOpt {
    fn cmp(&self, a: &T, b: &T) -> std::cmp::Ordering {
        a.cmp(b)
    }
    fn from_a(&self, m: &mut I, n: usize) -> EarlyOut {
        // println!("{:?}", m);
        // println!("move_a {}", n);
        m.move_a(n)
        // println!("{:?}\n", m);
    }
    fn from_b(&self, m: &mut I, n: usize) -> EarlyOut {
        // println!("{:?}", m);
        // println!("move_b {}", n);
        m.skip_b(n)
        // println!("{:?}\n", m);
    }
    fn collision(&self, m: &mut I) -> EarlyOut {
        // println!("{:?}", m);
        // println!("move_a 1");
        // println!("skip_b 1");
        m.skip_a(1)?;
        m.skip_b(1)
        // println!("{:?}\n", m);
    }
}

impl<'a, T: Ord, I: MergeState<T, T>> MergeOperation<'a, T, T, I> for SetXorOp {
    fn cmp(&self, a: &T, b: &T) -> std::cmp::Ordering {
        a.cmp(b)
    }
    fn from_a(&self, m: &mut I, n: usize) -> EarlyOut {
        // println!("{:?}", m);
        // println!("move_a {}", n);
        m.move_a(n)
        // println!("{:?}\n", m);
    }
    fn from_b(&self, m: &mut I, n: usize) -> EarlyOut {
        // println!("{:?}", m);
        // println!("move_b {}", n);
        m.move_b(n)
        // println!("{:?}\n", m);
    }
    fn collision(&self, m: &mut I) -> EarlyOut {
        // println!("{:?}", m);
        // println!("move_a 1");
        // println!("skip_b 1");
        m.skip_a(1)?;
        m.skip_b(1)
        // println!("{:?}\n", m);
    }
}

impl<T: Debug> Debug for ArraySet<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = self
            .0
            .iter()
            .map(|x| format!("{:?}", x))
            .collect::<Vec<String>>()
            .join(", ");
        write!(f, "{{{}}}", text)
    }
}

impl<T> ArraySet<T> {
    pub fn single(value: T) -> Self {
        Self(vec![value])
    }
    pub fn into_vec(self) -> Vec<T> {
        self.0
    }
    pub fn as_slice(&self) -> &[T] {
        &self.0
    }
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        &mut self.0
    }
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
    pub fn empty() -> Self {
        Self(Vec::new())
    }
}

impl<T> Default for ArraySet<T> {
    fn default() -> Self {
        ArraySet::empty()
    }
}

impl<T: Ord + Default + Copy + Debug> MeetSemilattice for ArraySet<T> {
    fn meet(&self, other: &Self) -> Self {
        self.intersection(other)
    }
}

impl<T: Ord + Default + Copy + Debug> JoinSemilattice for ArraySet<T> {
    fn join(&self, other: &Self) -> Self {
        self.union(other)
    }
}

// impl<T: Ord + Default + Copy + Debug> Lattice for ArraySet<T> {}

impl<T: Ord + Default + Copy + Debug> BitAnd for &ArraySet<T> {
    type Output = ArraySet<T>;
    fn bitand(self, rhs: Self) -> Self::Output {
        self.intersection(rhs)
    }
}

impl<T: Ord + Default + Copy + Debug> BitAndAssign for ArraySet<T> {
    fn bitand_assign(&mut self, rhs: Self) {
        self.intersection_with(&rhs)
    }
}

impl<T: Ord + Default + Copy + Debug> BitOr for &ArraySet<T> {
    type Output = ArraySet<T>;
    fn bitor(self, rhs: Self) -> Self::Output {
        self.union(rhs)
    }
}

impl<T: Ord + Default + Copy + Debug> BitOrAssign for ArraySet<T> {
    fn bitor_assign(&mut self, rhs: Self) {
        self.union_with(&rhs)
    }
}

impl<T: Ord + Default + Copy + Debug> BitXor for &ArraySet<T> {
    type Output = ArraySet<T>;
    fn bitxor(self, rhs: Self) -> Self::Output {
        self.xor(rhs)
    }
}

impl<T: Ord + Default + Copy + Debug> BitXorAssign for ArraySet<T> {
    fn bitxor_assign(&mut self, rhs: Self) {
        self.xor_with(&rhs)
    }
}

impl<T: Ord + Default + Copy + Debug> Sub for &ArraySet<T> {
    type Output = ArraySet<T>;
    fn sub(self, rhs: Self) -> Self::Output {
        self.difference(rhs)
    }
}

impl<T: Ord + Default + Copy + Debug> SubAssign for ArraySet<T> {
    fn sub_assign(&mut self, rhs: Self) {
        self.difference_with(&rhs)
    }
}

impl<T: Ord + Default + Copy + Debug> From<Vec<T>> for ArraySet<T> {
    fn from(vec: Vec<T>) -> Self {
        Self::from_vec(vec)
    }
}
impl<T: Ord + Default + Copy + Debug> std::iter::FromIterator<T> for ArraySet<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Self::from_vec(iter.into_iter().collect())
    }
}
impl<T> ArraySet<T> {
    pub fn shrink_to_fit(&mut self) {
        self.0.shrink_to_fit()
    }
}
impl<T: Ord> ArraySet<T> {
    pub fn is_disjoint(&self, that: &ArraySet<T>) -> bool {
        !BoolOpMergeState::merge(&self.0, &that.0, SetIntersectionOp())
    }

    pub fn is_subset(&self, that: &ArraySet<T>) -> bool {
        !BoolOpMergeState::merge(&self.0, &that.0, SetDiffOpt())
    }

    pub fn is_superset(&self, that: &ArraySet<T>) -> bool {
        that.is_subset(self)
    }
    pub fn contains(&self, value: &T) -> bool {
        self.0.binary_search(value).is_ok()
    }
}
impl<T: Ord + Default + Copy + Debug> ArraySet<T> {
    fn from_vec(vec: Vec<T>) -> Self {
        let mut vec = vec;
        vec.sort();
        vec.dedup();
        Self(vec)
    }

    pub fn union_with(&mut self, that: &ArraySet<T>) {
        InPlaceMergeState::merge(&mut self.0, &that.0, SetUnionOp());
    }

    pub fn union(&self, that: &ArraySet<T>) -> ArraySet<T> {
        ArraySet(VecMergeState::merge(&self.0, &that.0, SetUnionOp()))
    }

    pub fn intersection_with(&mut self, that: &ArraySet<T>) {
        InPlaceMergeState::merge(&mut self.0, &that.0, SetIntersectionOp());
    }

    pub fn intersection(&self, that: &ArraySet<T>) -> ArraySet<T> {
        ArraySet(VecMergeState::merge(&self.0, &that.0, SetIntersectionOp()))
    }

    pub fn xor_with(&mut self, that: &ArraySet<T>) {
        InPlaceMergeState::merge(&mut self.0, &that.0, SetXorOp());
    }

    pub fn xor(&self, that: &ArraySet<T>) -> ArraySet<T> {
        ArraySet(VecMergeState::merge(&self.0, &that.0, SetXorOp()))
    }

    pub fn difference_with(&mut self, that: &ArraySet<T>) {
        InPlaceMergeState::merge(&mut self.0, &that.0, SetDiffOpt());
    }

    pub fn difference(&self, that: &ArraySet<T>) -> ArraySet<T> {
        ArraySet(VecMergeState::merge(&self.0, &that.0, SetDiffOpt()))
    }

    pub fn insert(&mut self, that: T) {
        InPlaceMergeState::merge(&mut self.0, &[that], SetUnionOp());
    }

    pub fn remove(&mut self, that: &T) {
        InPlaceMergeState::merge(&mut self.0, &[*that], SetDiffOpt());
    }
}

// cargo asm abc::array_set::union_u32
pub fn union_u32(a: &mut Vec<u32>, b: &[u32]) {
    InPlaceMergeState::merge(a, b, SetUnionOp())
}

#[cfg(test)]
mod tests {
    use super::*;
    use quickcheck::*;
    use std::collections::BTreeSet;

    impl<T: Arbitrary + Ord + Copy + Default + Debug> quickcheck::Arbitrary for ArraySet<T> {
        fn arbitrary<G: Gen>(g: &mut G) -> Self {
            ArraySet::from_vec(Arbitrary::arbitrary(g))
        }
    }

    #[test]
    fn intersection_1() {
        let mut a: ArraySet<usize> = vec![0].into();
        let b: ArraySet<usize> = vec![].into();
        a.intersection_with(&b);
        println!("a {:?}", a);
        assert_eq!(a.into_vec(), vec![]);
    }

    fn binary_op(
        a: &ArraySet<i64>,
        b: &ArraySet<i64>,
        r: &ArraySet<i64>,
        op: impl Fn(bool, bool) -> bool,
    ) -> bool {
        let mut samples: BTreeSet<i64> = BTreeSet::new();
        samples.extend(a.as_slice().iter().cloned());
        samples.extend(b.as_slice().iter().cloned());
        samples.insert(std::i64::MIN);
        samples
            .iter()
            .all(|e| op(a.contains(e), b.contains(e)) == r.contains(e))
    }

    fn binary_property(
        a: &ArraySet<i64>,
        b: &ArraySet<i64>,
        r: bool,
        op: impl Fn(bool, bool) -> bool,
    ) -> bool {
        let mut samples: BTreeSet<i64> = BTreeSet::new();
        samples.extend(a.as_slice().iter().cloned());
        samples.extend(b.as_slice().iter().cloned());
        samples.insert(std::i64::MIN);
        if r {
            samples.iter().all(|e| {
                let expected = op(a.contains(e), b.contains(e));
                if !expected {
                    println!(
                        "{:?} is false at {:?}\na {:?}\nb {:?}\nr {:?}",
                        expected, e, a, b, r
                    );
                }
                expected
            })
        } else {
            samples.iter().any(|e| !op(a.contains(e), b.contains(e)))
        }
    }

    quickcheck! {

        fn is_disjoint_sample(a: ArraySet<i64>, b: ArraySet<i64>) -> bool {
            binary_property(&a, &b, a.is_disjoint(&b), |a, b| !(a & b))
        }

        fn is_subset_sample(a: ArraySet<i64>, b: ArraySet<i64>) -> bool {
            binary_property(&a, &b, a.is_subset(&b), |a, b| !a | b)
        }

        fn union_sample(a: ArraySet<i64>, b: ArraySet<i64>) -> bool {
            binary_op(&a, &b, &(&a | &b), |a, b| a | b)
        }

        fn intersection_sample(a: ArraySet<i64>, b: ArraySet<i64>) -> bool {
            binary_op(&a, &b, &(&a & &b), |a, b| a & b)
        }

        fn xor_sample(a: ArraySet<i64>, b: ArraySet<i64>) -> bool {
            binary_op(&a, &b, &(&a ^ &b), |a, b| a ^ b)
        }

        fn diff_sample(a: ArraySet<i64>, b: ArraySet<i64>) -> bool {
            binary_op(&a, &b, &(&a - &b), |a, b| a & !b)
        }

        fn union(a: BTreeSet<u32>, b: BTreeSet<u32>) -> bool {
            let mut a1: ArraySet<u32> = a.iter().cloned().collect();
            let b1: ArraySet<u32> = b.iter().cloned().collect();
            let r2 = &a1 | &b1;
            a1 |= b1;
            let expected: Vec<u32> = a.union(&b).cloned().collect();
            let actual: Vec<u32> = a1.into_vec();
            let actual2 = r2.into_vec();
            expected == actual && expected == actual2
        }

        fn intersection(a: BTreeSet<u32>, b: BTreeSet<u32>) -> bool {
            let mut a1: ArraySet<u32> = a.iter().cloned().collect();
            let b1: ArraySet<u32> = b.iter().cloned().collect();
            let r2 = &a1 & &b1;
            a1 &= b1;
            let expected: Vec<u32> = a.intersection(&b).cloned().collect();
            let actual: Vec<u32> = a1.into_vec();
            let actual2 = r2.into_vec();
            expected == actual && expected == actual2
        }

        fn xor(a: BTreeSet<u32>, b: BTreeSet<u32>) -> bool {
            let mut a1: ArraySet<u32> = a.iter().cloned().collect();
            let b1: ArraySet<u32> = b.iter().cloned().collect();
            let r2 = &a1 ^ &b1;
            a1 ^= b1;
            let expected: Vec<u32> = a.symmetric_difference(&b).cloned().collect();
            let actual: Vec<u32> = a1.into_vec();
            let actual2 = r2.into_vec();
            expected == actual && expected == actual2
        }

        fn difference(a: BTreeSet<u32>, b: BTreeSet<u32>) -> bool {
            let mut a1: ArraySet<u32> = a.iter().cloned().collect();
            let b1: ArraySet<u32> = b.iter().cloned().collect();
            let r2 = &a1 - &b1;
            a1 -= b1;
            let expected: Vec<u32> = a.difference(&b).cloned().collect();
            let actual: Vec<u32> = a1.into_vec();
            let actual2 = r2.into_vec();
            expected == actual && expected == actual2
        }

        fn is_disjoint(a: BTreeSet<u32>, b: BTreeSet<u32>) -> bool {
            let a1: ArraySet<u32> = a.iter().cloned().collect();
            let b1: ArraySet<u32> = b.iter().cloned().collect();
            let actual = a1.is_disjoint(&b1);
            let expected = a.is_disjoint(&b);
            expected == actual
        }

        fn is_subset(a: BTreeSet<u32>, b: BTreeSet<u32>) -> bool {
            let a1: ArraySet<u32> = a.iter().cloned().collect();
            let b1: ArraySet<u32> = b.iter().cloned().collect();
            let actual = a1.is_subset(&b1);
            let expected = a.is_subset(&b);
            expected == actual
        }

        fn contains(a: BTreeSet<u32>, b: u32) -> bool {
            let a1: ArraySet<u32> = a.iter().cloned().collect();
            let expected = a.contains(&b);
            let actual = a1.contains(&b);
            expected == actual
        }
    }
}
