use std::cmp::Ordering;

/// The read part of the merge state that is needed for the binary merge algorithm
/// it just needs random access for the remainder of a and b
pub(crate) trait MergeStateRead<A, B> {
    /// The remaining data in a
    fn a_slice(&self) -> &[A];
    /// The remaining data in b
    fn b_slice(&self) -> &[B];
}

pub(crate) trait MergeOperation<A, B, M: MergeStateRead<A, B>> {
    fn from_a(&self, m: &mut M, n: usize);
    fn from_b(&self, m: &mut M, n: usize);
    fn collision(&self, m: &mut M);
    fn cmp(&self, a: &A, b: &B) -> Ordering;
    /// merge `an` elements from a and `bn` elements from b into the result
    fn merge0(&self, m: &mut M, an: usize, bn: usize) {
        if an == 0 {
            if bn > 0 {
                self.from_b(m, bn);
            }
        } else if bn == 0 {
            if an > 0 {
                self.from_a(m, an);
            }
        } else {
            // neither a nor b are 0
            let am: usize = an / 2;
            // pick the center element of a and find the corresponding one in b using binary search
            let a = &m.a_slice()[am];
            match m.b_slice()[0..bn].binary_search_by(|b| self.cmp(a, b).reverse()) {
                Result::Ok(bm) => {
                    // same elements. bm is the index corresponding to am
                    // merge everything below am with everything below the found element bm
                    self.merge0(m, am, bm);
                    // add the elements a(am) and b(bm)
                    self.collision(m);
                    // merge everything above a(am) with everything above the found element
                    self.merge0(m, an - am - 1, bn - bm - 1);
                }
                Result::Err(bi) => {
                    // not found. bi is the insertion point
                    // merge everything below a(am) with everything below the found insertion point bi
                    self.merge0(m, am, bi);
                    // add a(am)
                    self.from_a(m, 1);
                    // everything above a(am) with everything above the found insertion point
                    self.merge0(m, an - am - 1, bn - bi);
                }
            }
        }
    }
    fn merge(&self, m: &mut M) {
        let a1 = m.a_slice().len();
        let b1 = m.b_slice().len();
        self.merge0(m, a1, b1);
    }
}

/// Basically a convenient to use bool to allow aborting a piece of code early using ?
pub(crate) type EarlyOut = Option<()>;

///
/// A minimum comparison merge operation. Not 100% sure if this is actually minimum comparison,
/// since proving this is beyond my ability. But it is optimal for many common cases.
///
pub(crate) trait ShortcutMergeOperation<A, B, M: MergeStateRead<A, B>> {
    fn from_a(&self, m: &mut M, n: usize) -> EarlyOut;
    fn from_b(&self, m: &mut M, n: usize) -> EarlyOut;
    fn collision(&self, m: &mut M) -> EarlyOut;
    fn cmp(&self, a: &A, b: &B) -> Ordering;
    /// merge `an` elements from a and `bn` elements from b into the result
    fn merge0(&self, m: &mut M, an: usize, bn: usize) -> EarlyOut {
        if an == 0 {
            if bn > 0 {
                self.from_b(m, bn)?
            }
        } else if bn == 0 {
            if an > 0 {
                self.from_a(m, an)?
            }
        } else {
            // neither a nor b are 0
            let am: usize = an / 2;
            // pick the center element of a and find the corresponding one in b using binary search
            let a = &m.a_slice()[am];
            match m.b_slice()[0..bn].binary_search_by(|b| self.cmp(a, b).reverse()) {
                Result::Ok(bm) => {
                    // same elements. bm is the index corresponding to am
                    // merge everything below am with everything below the found element bm
                    self.merge0(m, am, bm)?;
                    // add the elements a(am) and b(bm)
                    self.collision(m)?;
                    // merge everything above a(am) with everything above the found element
                    self.merge0(m, an - am - 1, bn - bm - 1)?;
                }
                Result::Err(bi) => {
                    // not found. bi is the insertion point
                    // merge everything below a(am) with everything below the found insertion point bi
                    self.merge0(m, am, bi)?;
                    // add a(am)
                    self.from_a(m, 1)?;
                    // everything above a(am) with everything above the found insertion point
                    self.merge0(m, an - am - 1, bn - bi)?;
                }
            }
        }
        Some(())
    }
    fn merge(&self, m: &mut M) {
        let a1 = m.a_slice().len();
        let b1 = m.b_slice().len();
        self.merge0(m, a1, b1);
    }
}
