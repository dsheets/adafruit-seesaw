use core::ops::{Index, IndexMut, Range, RangeFrom};

pub type Reg = [u8; 2];

pub struct RegValue<const N: usize>
where
    [(); 2 + N]: Sized,
{
    data: [u8; 2 + N],
}

impl<const N: usize> RegValue<N>
where
    [(); 2 + N]: Sized,
{
    pub fn new(reg: &Reg) -> Self {
        let mut data = [0; 2 + N];
        data[0..2].copy_from_slice(reg);
        Self { data }
    }

    pub fn with_bytes(mut self, bytes: &[u8]) -> Self {
        let len = bytes.len();
        if len > N {
            panic!("can't copy {len} bytes into {N} bytes")
        }

        self.data[2..].copy_from_slice(bytes);
        self
    }

    pub fn buffer(&self) -> &[u8] {
        &self.data
    }
}

fn shift_range(range: Range<usize>, k: usize) -> Range<usize> {
    Range {
        start: k + range.start,
        end: k + range.end,
    }
}

fn shift_range_from(range_from: RangeFrom<usize>, k: usize) -> RangeFrom<usize> {
    RangeFrom {
        start: k + range_from.start,
    }
}

impl<const N: usize> Index<Range<usize>> for RegValue<N>
where
    [(); 2 + N]: Sized,
{
    type Output = <[u8; N] as Index<Range<usize>>>::Output;

    fn index(&self, index: Range<usize>) -> &Self::Output {
        self.data.index(shift_range(index, 2))
    }
}

impl<const N: usize> IndexMut<Range<usize>> for RegValue<N>
where
    [(); 2 + N]: Sized,
{
    fn index_mut(&mut self, index: Range<usize>) -> &mut Self::Output {
        self.data.index_mut(shift_range(index, 2))
    }
}

impl<const N: usize> Index<RangeFrom<usize>> for RegValue<N>
where
    [(); 2 + N]: Sized,
{
    type Output = <[u8; N] as Index<RangeFrom<usize>>>::Output;

    fn index(&self, index: RangeFrom<usize>) -> &Self::Output {
        self.data.index(shift_range_from(index, 2))
    }
}

impl<const N: usize> IndexMut<RangeFrom<usize>> for RegValue<N>
where
    [(); 2 + N]: Sized,
{
    fn index_mut(&mut self, index: RangeFrom<usize>) -> &mut Self::Output {
        self.data.index_mut(shift_range_from(index, 2))
    }
}
