use super::*;
use std::fmt;
use std::fmt::Display;
use std::marker::PhantomData;
use std::ops::BitAnd;
use std::ops::BitOr;
use std::ops::Not;
use std::slice::*;

impl<T: Ord + Eq> IntoIterator for IntervalSeq<T> {
    type Item = T;
    type IntoIter = ::std::vec::IntoIter<T>;
    fn into_iter(self: IntervalSeq<T>) -> Self::IntoIter {
        self.values.into_iter()
    }
}

trait IntervalSet<T: Eq> {
    fn is_empty(&self) -> bool;
    // fn is_contiguous(&self) -> bool;
    // fn hull(&self) -> Interval<T>;
    fn at(&self, value: T) -> bool;
    fn above(&self, value: T) -> bool;
    fn below(&self, value: T) -> bool;
    fn above_all(&self) -> bool;
    fn below_all(&self) -> bool;
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
#[repr(u8)]
enum Kind {
    K00 = 0,
    K01 = 1,
    K10 = 2,
    K11 = 3,
}

impl BitAnd for Kind {
    type Output = Kind;
    fn bitand(self: Kind, that: Kind) -> Kind {
        Kind::from_u8((self as u8) & (that as u8))
    }
}

impl BitOr for Kind {
    type Output = Kind;
    fn bitor(self: Kind, that: Kind) -> Kind {
        Kind::from_u8((self as u8) | (that as u8))
    }
}

impl Not for Kind {
    type Output = Kind;
    fn not(self: Kind) -> Kind {
        Kind::from_u8(self as u8)
    }
}

impl Kind {
    fn from_u8(value: u8) -> Kind {
        match value & 3 {
            0 => Kind::K00,
            1 => Kind::K01,
            2 => Kind::K10,
            3 => Kind::K11,
            _ => panic!(),
        }
    }
    fn value_at(self: &Kind) -> bool {
        match self {
            Kind::K10 | Kind::K11 => true,
            _ => false,
        }
    }

    fn value_above(self: &Kind) -> bool {
        match self {
            Kind::K01 | Kind::K11 => true,
            _ => false,
        }
    }
}

#[derive(Debug)]
pub struct IntervalSeq<T: Ord> {
    below_all: bool,
    values: Vec<T>,
    kinds: Vec<Kind>,
}

impl<T: Ord + Copy> IntervalSeq<T> {
    fn singleton(below_all: bool, value: T, kind: Kind) -> IntervalSeq<T> {
        IntervalSeq {
            below_all,
            values: vec![value],
            kinds: vec![kind],
        }
    }
    fn from_to(from: T, fk: Kind, to: T, tk: Kind) -> IntervalSeq<T> {
        IntervalSeq {
            below_all: false,
            values: vec![from, to],
            kinds: vec![fk, tk],
        }
    }
    pub fn from_bool(value: bool) -> IntervalSeq<T> {
        if value {
            IntervalSeq::all()
        } else {
            IntervalSeq::empty()
        }
    }
    pub fn empty() -> IntervalSeq<T> {
        IntervalSeq {
            below_all: false,
            kinds: Vec::new(),
            values: Vec::new(),
        }
    }
    pub fn all() -> IntervalSeq<T> {
        IntervalSeq {
            below_all: true,
            kinds: Vec::new(),
            values: Vec::new(),
        }
    }
    pub fn at(value: T) -> IntervalSeq<T> {
        IntervalSeq::singleton(false, value, Kind::K10)
    }
    pub fn except(value: T) -> IntervalSeq<T> {
        IntervalSeq::singleton(true, value, Kind::K01)
    }
    pub fn below(value: T) -> IntervalSeq<T> {
        IntervalSeq::singleton(true, value, Kind::K00)
    }
    pub fn at_or_below(value: T) -> IntervalSeq<T> {
        IntervalSeq::singleton(true, value, Kind::K10)
    }
    pub fn at_or_above(value: T) -> IntervalSeq<T> {
        IntervalSeq::singleton(false, value, Kind::K11)
    }
    pub fn above(value: T) -> IntervalSeq<T> {
        IntervalSeq::singleton(false, value, Kind::K01)
    }
    pub fn from_interval(value: Interval<T>) -> IntervalSeq<T> {
        match (value.lower_bound(), value.upper_bound()) {
            (Bound::Closed(a), Bound::Closed(b)) if a == b => IntervalSeq::at(a),
            (Bound::Unbound, Bound::Open(x)) => IntervalSeq::below(x),
            (Bound::Unbound, Bound::Closed(x)) => IntervalSeq::at_or_below(x),
            (Bound::Open(x), Bound::Unbound) => IntervalSeq::above(x),
            (Bound::Closed(x), Bound::Unbound) => IntervalSeq::at_or_above(x),
            (Bound::Closed(a), Bound::Closed(b)) => {
                IntervalSeq::from_to(a, Kind::K11, b, Kind::K10)
            }
            (Bound::Closed(a), Bound::Open(b)) => IntervalSeq::from_to(a, Kind::K11, b, Kind::K00),
            (Bound::Open(a), Bound::Closed(b)) => IntervalSeq::from_to(a, Kind::K01, b, Kind::K10),
            (Bound::Open(a), Bound::Open(b)) => IntervalSeq::from_to(a, Kind::K01, b, Kind::K00),
            (Bound::Unbound, Bound::Unbound) => IntervalSeq::all(),
            (Bound::Empty, Bound::Empty) => IntervalSeq::empty(),
            _ => IntervalSeq::empty(),
        }
    }
}

struct IntervalIterator<'a, T: Ord> {
    values: &'a Vec<T>,
    kinds: &'a Vec<Kind>,
    lower: Option<Bound<T>>,
    i: usize,
}

/**
 * Provide the abiltiy to read from an OpState
 */
trait Read<T: Ord> {
    fn a(&self) -> &IntervalSeq<T>;
    fn b(&self) -> &IntervalSeq<T>;
}

trait Builder<T: Ord> {
    fn copy_from(&mut self, src: &IntervalSeq<T>, i0: usize, i1: usize) -> ();
    fn flip_from(&mut self, src: &IntervalSeq<T>, i0: usize, i1: usize) -> ();
    fn append(&mut self, value: T, kind: Kind) -> ();
}

impl<T: Ord + Clone> Builder<T> for IntervalSeq<T> {
    fn copy_from(&mut self, src: &IntervalSeq<T>, i0: usize, i1: usize) -> () {
        self.values.extend_from_slice(&src.values[i0..i1]);
        self.kinds.extend_from_slice(&src.kinds[i0..i1]);
    }
    fn flip_from(&mut self, src: &IntervalSeq<T>, i0: usize, i1: usize) -> () {
        self.values.extend_from_slice(&src.values[i0..i1]);
        self.kinds.extend_from_slice(&src.kinds[i0..i1]);
        for i in i0..i1 {
            self.kinds[i] = !self.kinds[i]
        }
    }
    fn append(&mut self, value: T, kind: Kind) -> () {
        self.values.push(value);
        self.kinds.push(kind);
    }
}

/**
 * State of an operation. Parametrized on the result type and the operation kind.
 */
struct OpState<T: Ord, R, K> {
    a: IntervalSeq<T>,
    b: IntervalSeq<T>,
    r: R,
    k: PhantomData<K>,
}

/**
 * Read impl for OpState
 */
impl<T: Ord, R, K> Read<T> for OpState<T, R, K> {
    fn a(&self) -> &IntervalSeq<T> {
        &self.a
    }
    fn b(&self) -> &IntervalSeq<T> {
        &self.b
    }
}

impl<T: Ord + Clone, K> Builder<T> for OpState<T, IntervalSeq<T>, K> {
    fn copy_from(&mut self, src: &IntervalSeq<T>, i0: usize, i1: usize) -> () {
        self.r.copy_from(src, i0, i1)
    }
    fn flip_from(&mut self, src: &IntervalSeq<T>, i0: usize, i1: usize) -> () {
        self.r.flip_from(src, i0, i1)
    }
    fn append(&mut self, value: T, kind: Kind) -> () {
        let current = self.r.above_all();
        // do not append redundant values
        if (current && kind != Kind::K11) || (!current && kind != Kind::K00) {
            self.r.append(value, kind)
        }
    }
}

/**
 * Basic binary operation.
 */
trait BinaryOperation<T: Ord>: Read<T> {
    fn from_a(&mut self, a0: usize, a1: usize, b: bool) -> ();
    fn from_b(&mut self, a: bool, b0: usize, b1: usize) -> ();
    fn collision(&mut self, ai: usize, bi: usize) -> ();
    fn merge0(&mut self, a0: usize, a1: usize, b0: usize, b1: usize) -> () {
        if a0 == a1 {
            self.from_b(self.a().below_index(a0), b0, b1)
        } else if b0 == b1 {
            self.from_a(a0, a1, self.b().below_index(b0))
        } else {
            let am: usize = (a0 + a1) / 2;
            match self.b().values[b0..b1].binary_search(&self.a().values[am]) {
                Result::Ok(bm) => {
                    // same elements. bm is the index corresponding to am
                    // merge everything below a(am) with everything below the found element
                    self.merge0(a0, am, b0, bm);
                    // add the elements a(am) and b(bm)
                    self.collision(am, bm);
                    // merge everything above a(am) with everything above the found element
                    self.merge0(am + 1, a1, bm + 1, b1);
                }
                Result::Err(bi) => {
                    // not found. bi is the insertion point
                    // merge everything below a(am) with everything below the found insertion point
                    self.merge0(a0, am, b0, bi);
                    // add a(am)
                    self.from_a(am, am + 1, self.b().below_index(bi));
                    // everything above a(am) with everything above the found insertion point
                    self.merge0(am + 1, a1, bi, b1);
                }
            }
        }
    }
}

struct AndOperation {}

impl<T: Ord + Copy> BinaryOperation<T> for OpState<T, IntervalSeq<T>, AndOperation> {
    fn from_a(&mut self, a0: usize, a1: usize, b: bool) -> () {
        if b {
            self.r.copy_from(&self.a, a0, a1)
        }
    }
    fn from_b(&mut self, a: bool, b0: usize, b1: usize) -> () {
        if a {
            self.r.copy_from(&self.b, b0, b1)
        }
    }
    fn collision(&mut self, ai: usize, bi: usize) -> () {
        let value = self.a.values[ai];
        let kind = self.a.kinds[ai] & self.b.kinds[bi];
        self.r.append(value, kind)
    }
}

impl<T: Ord + Copy> BitAnd for IntervalSeq<T> {
    type Output = IntervalSeq<T>;
    fn bitand(self: IntervalSeq<T>, that: IntervalSeq<T>) -> IntervalSeq<T> {
        let r_below_all = IntervalSeq::from_bool(self.below_all & that.below_all);
        let mut r: OpState<T, IntervalSeq<T>, AndOperation> = OpState {
            a: self,
            b: that,
            r: r_below_all,
            k: PhantomData {},
        };
        r.merge0(0, r.a.values.len(), 0, r.b.values.len());
        r.r
    }
}

struct OrOperation {}

impl<T: Ord + Copy> BinaryOperation<T> for OpState<T, IntervalSeq<T>, OrOperation> {
    fn from_a(&mut self, a0: usize, a1: usize, b: bool) -> () {
        if !b {
            self.r.copy_from(&self.a, a0, a1)
        }
    }
    fn from_b(&mut self, a: bool, b0: usize, b1: usize) -> () {
        if !a {
            self.r.copy_from(&self.b, b0, b1)
        }
    }
    fn collision(&mut self, ai: usize, bi: usize) -> () {
        let value = self.a.values[ai];
        let kind = self.a.kinds[ai] | self.b.kinds[bi];
        self.r.append(value, kind)
    }
}

impl<T: Ord + Copy> BitOr for IntervalSeq<T> {
    type Output = IntervalSeq<T>;
    fn bitor(self: IntervalSeq<T>, that: IntervalSeq<T>) -> IntervalSeq<T> {
        let r_below_all = IntervalSeq::from_bool(self.below_all | that.below_all);
        let mut r: OpState<T, IntervalSeq<T>, OrOperation> = OpState {
            a: self,
            b: that,
            r: r_below_all,
            k: PhantomData {},
        };
        r.merge0(0, r.a.values.len(), 0, r.b.values.len());
        r.r
    }
}

impl<T: Ord + FromStr + Display + Copy> FromStr for IntervalSeq<T> {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let _intervals: Result<Vec<Interval<T>>, _> =
            s.split(";").map(|x| x.parse::<Interval<T>>()).collect();
        Ok(IntervalSeq::empty())
    }
}

impl<'a, T: Ord + Copy> IntervalIterator<'a, T> {
    fn next_interval(self: &mut IntervalIterator<'a, T>) -> Option<Interval<T>> {
        if self.i < self.values.len() {
            let value = self.values[self.i];
            let kind = self.kinds[self.i];
            self.i += 1;
            match (self.lower, kind) {
                (Option::None, Kind::K10) => {
                    self.lower = None;
                    Some(Interval::Point(value))
                }
                (Option::None, Kind::K11) => {
                    self.lower = Some(Bound::Closed(value));
                    None
                }
                (Option::None, Kind::K01) => {
                    self.lower = Some(Bound::Open(value));
                    None
                }
                (Option::None, _) => panic!(),

                (Option::Some(lower), Kind::K01) => {
                    let upper = Bound::Open(value);
                    self.lower = Some(upper);
                    Some(Interval::from_bounds(lower, upper))
                }
                (Option::Some(lower), Kind::K00) => {
                    let upper = Bound::Open(value);
                    self.lower = None;
                    Some(Interval::from_bounds(lower, upper))
                }
                (Option::Some(lower), Kind::K10) => {
                    let upper = Bound::Closed(value);
                    self.lower = None;
                    Some(Interval::from_bounds(lower, upper))
                }
                (Option::Some(_), _) => {
                    panic!();
                }
            }
        } else {
            match self.lower {
                Some(lower) => {
                    self.lower = None;
                    Some(Interval::from_bounds(lower, Bound::Unbound))
                }
                None => None,
            }
        }
    }
}

impl<'a, T: Ord + Copy> std::iter::Iterator for IntervalIterator<'a, T> {
    type Item = Interval<T>;

    fn next(self: &mut IntervalIterator<'a, T>) -> Option<Interval<T>> {
        let has_next = self.i < self.values.len() || self.lower.is_some();
        match IntervalIterator::next_interval(self) {
            Some(x) => Some(x),
            None if has_next => IntervalIterator::next(self),
            _ => None,
        }
    }
}

impl<T: Ord> IntervalSeq<T> {
    fn below_index(self: &IntervalSeq<T>, index: usize) -> bool {
        if index == 0 {
            self.below_all
        } else {
            self.kinds[index - 1].value_above()
        }
    }

    fn intervals(self: &IntervalSeq<T>) -> IntervalIterator<T> {
        IntervalIterator {
            i: 0,
            kinds: &self.kinds,
            values: &self.values,
            lower: if self.below_all {
                Some(Bound::Unbound)
            } else {
                None
            },
        }
    }

    fn edges(self: IntervalSeq<T>) -> Vec<T> {
        self.values
    }
}

impl<T: Eq + Ord + Copy + Display> Display for IntervalSeq<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let text: String = self
            .intervals()
            .map(|x| format!("{}", x))
            .collect::<Vec<String>>()
            .join("; ");
        write!(f, "{}", text)
    }
}

impl<T: Eq + Ord> IntervalSet<T> for IntervalSeq<T> {
    fn is_empty(self: &IntervalSeq<T>) -> bool {
        !self.below_all && self.values.is_empty()
    }

    fn at(self: &IntervalSeq<T>, value: T) -> bool {
        match self.values.as_slice().binary_search(&value) {
            Ok(index) => self.kinds[index].value_at(),
            Err(index) => self.below_index(index),
        }
    }
    fn above(self: &IntervalSeq<T>, value: T) -> bool {
        match self.values.as_slice().binary_search(&value) {
            Ok(index) => self.kinds[index].value_above(),
            Err(index) => self.below_index(index),
        }
    }
    fn below(self: &IntervalSeq<T>, value: T) -> bool {
        match self.values.as_slice().binary_search(&value) {
            Ok(index) => {
                if index > 0 {
                    self.kinds[index - 1].value_above()
                } else {
                    self.below_all
                }
            }
            Err(index) => self.below_index(index),
        }
    }
    fn below_all(self: &IntervalSeq<T>) -> bool {
        self.below_all
    }
    fn above_all(self: &IntervalSeq<T>) -> bool {
        self.kinds
            .last()
            .map_or(self.below_all(), |k| k.value_above())
    }
}
