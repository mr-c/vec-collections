use crate::array_map::SliceIterator;
use crate::binary_merge::{EarlyOut, MergeStateMod, ShortcutMergeOperation};
use crate::dedup::SortAndDedup;
use crate::merge_state::{
    BoolOpMergeState, InPlaceMergeState, UnsafeInPlaceMergeState, VecMergeState,
};
use std::collections::BTreeSet;
use std::fmt::Debug;
use std::iter::FromIterator;
use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Sub, SubAssign};

struct SetUnionOp;
struct SetIntersectionOp;
struct SetXorOp;
struct SetDiffOpt;

#[derive(Clone, Hash)]
pub struct ArraySet<T>(Vec<T>);

impl<T: Ord, I: MergeStateMod<T, T>> ShortcutMergeOperation<T, T, I> for SetUnionOp {
    fn cmp(&self, a: &T, b: &T) -> std::cmp::Ordering {
        a.cmp(b)
    }
    fn from_a(&self, m: &mut I, n: usize) -> EarlyOut {
        m.move_a(n)
    }
    fn from_b(&self, m: &mut I, n: usize) -> EarlyOut {
        m.move_b(n)
    }
    fn collision(&self, m: &mut I) -> EarlyOut {
        m.move_a(1)?;
        m.skip_b(1)
    }
}

impl<T: Ord, I: MergeStateMod<T, T>> ShortcutMergeOperation<T, T, I> for SetIntersectionOp {
    fn cmp(&self, a: &T, b: &T) -> std::cmp::Ordering {
        a.cmp(b)
    }
    fn from_a(&self, m: &mut I, n: usize) -> EarlyOut {
        m.skip_a(n)
    }
    fn from_b(&self, m: &mut I, n: usize) -> EarlyOut {
        m.skip_b(n)
    }
    fn collision(&self, m: &mut I) -> EarlyOut {
        m.move_a(1)?;
        m.skip_b(1)
    }
}

impl<T: Ord, I: MergeStateMod<T, T>> ShortcutMergeOperation<T, T, I> for SetDiffOpt {
    fn cmp(&self, a: &T, b: &T) -> std::cmp::Ordering {
        a.cmp(b)
    }
    fn from_a(&self, m: &mut I, n: usize) -> EarlyOut {
        m.move_a(n)
    }
    fn from_b(&self, m: &mut I, n: usize) -> EarlyOut {
        m.skip_b(n)
    }
    fn collision(&self, m: &mut I) -> EarlyOut {
        m.skip_a(1)?;
        m.skip_b(1)
    }
}

impl<T: Ord, I: MergeStateMod<T, T>> ShortcutMergeOperation<T, T, I> for SetXorOp {
    fn cmp(&self, a: &T, b: &T) -> std::cmp::Ordering {
        a.cmp(b)
    }
    fn from_a(&self, m: &mut I, n: usize) -> EarlyOut {
        m.move_a(n)
    }
    fn from_b(&self, m: &mut I, n: usize) -> EarlyOut {
        m.move_b(n)
    }
    fn collision(&self, m: &mut I) -> EarlyOut {
        m.skip_a(1)?;
        m.skip_b(1)
    }
}

impl<T: Debug> Debug for ArraySet<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_set().entries(self.0.iter()).finish()
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
    pub fn len(&self) -> usize {
        self.0.len()
    }
    pub fn shrink_to_fit(&mut self) {
        self.0.shrink_to_fit()
    }
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
    pub fn empty() -> Self {
        Self(Vec::new())
    }
    pub fn iter(&self) -> SliceIterator<T> {
        SliceIterator(self.as_slice())
    }
}

impl<T> Default for ArraySet<T> {
    fn default() -> Self {
        ArraySet::empty()
    }
}

impl<T: Ord> BitAndAssign for ArraySet<T> {
    fn bitand_assign(&mut self, that: Self) {
        UnsafeInPlaceMergeState::merge_shortcut(&mut self.0, that.0, SetIntersectionOp);
    }
}

impl<T: Ord> BitOrAssign for ArraySet<T> {
    fn bitor_assign(&mut self, that: Self) {
        UnsafeInPlaceMergeState::merge_shortcut(&mut self.0, that.0, SetUnionOp);
    }
}

impl<T: Ord> BitXorAssign for ArraySet<T> {
    fn bitxor_assign(&mut self, that: Self) {
        UnsafeInPlaceMergeState::merge_shortcut(&mut self.0, that.0, SetXorOp);
    }
}

impl<T: Ord> SubAssign for ArraySet<T> {
    fn sub_assign(&mut self, that: Self) {
        UnsafeInPlaceMergeState::merge_shortcut(&mut self.0, that.0, SetDiffOpt);
    }
}

impl<T: Ord + Clone> BitAnd for &ArraySet<T> {
    type Output = ArraySet<T>;
    fn bitand(self, rhs: Self) -> Self::Output {
        self.intersection(rhs)
    }
}

impl<T: Ord> BitAnd for ArraySet<T> {
    type Output = ArraySet<T>;
    fn bitand(mut self, that: Self) -> Self::Output {
        self &= that;
        self
    }
}

impl<T: Ord + Clone> BitOr for &ArraySet<T> {
    type Output = ArraySet<T>;
    fn bitor(self, rhs: Self) -> Self::Output {
        self.union(rhs)
    }
}

impl<T: Ord> BitOr for ArraySet<T> {
    type Output = ArraySet<T>;
    fn bitor(mut self, that: Self) -> Self::Output {
        self |= that;
        self
    }
}

impl<T: Ord + Clone> BitXor for &ArraySet<T> {
    type Output = ArraySet<T>;
    fn bitxor(self, rhs: Self) -> Self::Output {
        self.xor(rhs)
    }
}

impl<T: Ord> BitXor for ArraySet<T> {
    type Output = ArraySet<T>;
    fn bitxor(mut self, that: Self) -> Self::Output {
        self ^= that;
        self
    }
}

impl<T: Ord + Clone> Sub for &ArraySet<T> {
    type Output = ArraySet<T>;
    fn sub(self, rhs: Self) -> Self::Output {
        self.difference(rhs)
    }
}

impl<T: Ord> Sub for ArraySet<T> {
    type Output = ArraySet<T>;
    fn sub(mut self, that: Self) -> Self::Output {
        self -= that;
        self
    }
}

impl<T: Ord> From<Vec<T>> for ArraySet<T> {
    fn from(vec: Vec<T>) -> Self {
        Self::from_vec(vec)
    }
}

impl<T: Ord> From<BTreeSet<T>> for ArraySet<T> {
    fn from(value: BTreeSet<T>) -> Self {
        Self(value.into_iter().collect())
    }
}

impl<T: Ord> FromIterator<T> for ArraySet<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut iter = iter.into_iter();
        let mut agg = SortAndDedup::<T>::new();
        while let Some(x) = iter.next() {
            agg.push(x);
        }
        Self::from_vec(agg.result())
    }
}

impl<T: Ord> Extend<T> for ArraySet<T> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        *self &= Self::from_iter(iter);
    }
}

impl<'a, T: 'a + Ord + Copy> Extend<&'a T> for ArraySet<T> {
    fn extend<I: IntoIterator<Item = &'a T>>(&mut self, iter: I) {
        self.extend(iter.into_iter().cloned())
    }
}

impl<T: Ord> ArraySet<T> {
    pub fn is_disjoint(&self, that: &ArraySet<T>) -> bool {
        !BoolOpMergeState::merge(&self.0, &that.0, SetIntersectionOp)
    }

    pub fn is_subset(&self, that: &ArraySet<T>) -> bool {
        !BoolOpMergeState::merge(&self.0, &that.0, SetDiffOpt)
    }

    pub fn is_superset(&self, that: &ArraySet<T>) -> bool {
        that.is_subset(self)
    }
    pub fn contains(&self, value: &T) -> bool {
        self.0.binary_search(value).is_ok()
    }

    pub fn retain<F: FnMut(&T) -> bool>(&mut self, f: F) {
        self.0.retain(f)
    }

    fn from_vec(vec: Vec<T>) -> Self {
        let mut vec = vec;
        vec.sort();
        vec.dedup();
        Self(vec)
    }
}

impl<T: Ord + Clone> ArraySet<T> {
    pub fn union(&self, that: &ArraySet<T>) -> ArraySet<T> {
        ArraySet(VecMergeState::merge(&self.0, &that.0, SetUnionOp))
    }

    pub fn intersection(&self, that: &ArraySet<T>) -> ArraySet<T> {
        ArraySet(VecMergeState::merge(&self.0, &that.0, SetIntersectionOp))
    }

    pub fn xor(&self, that: &ArraySet<T>) -> ArraySet<T> {
        ArraySet(VecMergeState::merge(&self.0, &that.0, SetXorOp))
    }

    pub fn difference(&self, that: &ArraySet<T>) -> ArraySet<T> {
        ArraySet(VecMergeState::merge(&self.0, &that.0, SetDiffOpt))
    }

    pub fn insert(&mut self, that: T) {
        match self.0.binary_search(&that) {
            Ok(index) => self.0[index] = that,
            Err(index) => self.0.insert(index, that),
        }
    }

    pub fn remove(&mut self, that: &T) {
        match self.0.binary_search(&that) {
            Ok(index) => {
                self.0.remove(index);
            }
            _ => {}
        };
    }
}

// impl<T: Ord + Default + Copy> ArraySet<T> {
//     pub fn union_with(&mut self, that: &ArraySet<T>) {
//         InPlaceMergeState::merge(&mut self.0, &that.0, SetUnionOp());
//     }

//     pub fn intersection_with(&mut self, that: &ArraySet<T>) {
//         InPlaceMergeState::merge(&mut self.0, &that.0, SetIntersectionOp());
//     }

//     pub fn xor_with(&mut self, that: &ArraySet<T>) {
//         InPlaceMergeState::merge(&mut self.0, &that.0, SetIntersectionOp());
//     }

//     pub fn difference_with(&mut self, that: &ArraySet<T>) {
//         InPlaceMergeState::merge(&mut self.0, &that.0, SetDiffOpt());
//     }
// }

// cargo asm vec_set::array_set::union_u32
pub fn union_u32(a: &mut Vec<u32>, b: &[u32]) {
    InPlaceMergeState::merge(a, b, SetUnionOp)
}

#[cfg(test)]
mod tests {
    use super::*;
    use quickcheck::*;
    use std::collections::BTreeSet;

    impl<T: Arbitrary + Ord + Copy + Default + Debug> Arbitrary for ArraySet<T> {
        fn arbitrary<G: Gen>(g: &mut G) -> Self {
            ArraySet::from_vec(Arbitrary::arbitrary(g))
        }
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
