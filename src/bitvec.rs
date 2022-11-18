pub struct Bitvec {
    data: Vec<u64>,
    len: usize,
}

impl Bitvec {
    pub fn new(size: usize) -> Self {
        let n = (size + 63) / 64;
        Bitvec {
            data: vec![0; n],
            len: size,
        }
    }

    pub fn at(&self, n: usize) -> bool {
        assert!(n < self.len);
        (self.data[n / 64] >> (n % 64)) & 1 == 1
    }

    pub fn flip(&mut self, n: usize) {
        assert!(n < self.len);
        self.data[n / 64] ^= 1 << (n % 64);
    }

    pub fn set(&mut self, n: usize, bit: bool) {
        if self.at(n) != bit {
            self.flip(n);
        }
    }

    pub fn push(&mut self, bit: bool) {
        if self.len % 64 == 0 {
            self.data.push(0);
        }
        self.len += 1;
        self.set(self.len - 1, bit);
    }
}
