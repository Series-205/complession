use super::bitvec;

pub struct SuffixArray<'a, T = u8> {
    pub sa: Vec<usize>,
    pub s: &'a [T],
}

impl<'a> SuffixArray<'a> {
    const NIL: usize = usize::MAX;
    const SIGMA: usize = 256;
    pub fn new(s: &'a [u8]) -> Self {
        let sa = Self::sa_is(&s, Self::SIGMA);

        Self { sa, s }
    }

    fn is_lms(is_s: &bitvec::Bitvec, i: usize) -> bool {
        i > 0 && !is_s.at(i - 1) && is_s.at(i)
    }

    fn sa_is<T>(s: &[T], sigma: usize) -> Vec<usize>
    where
        T: Ord + Into<usize> + Copy,
    {
        let n = s.len();
        // L or S
        let mut is_s = bitvec::Bitvec::new(n);
        for i in (0..n).rev() {
            if i == n - 1 || s[i] < s[i + 1] {
                is_s.set(i, true);
            } else if s[i] == s[i + 1] {
                is_s.set(i, is_s.at(i + 1));
            }
        }
        let mut lms = Vec::new();
        for i in 0..n {
            if Self::is_lms(&is_s, i) {
                lms.push(i);
            }
        }

        // induced sort の結果から LMS の suffix のみ取り出す
        let sa_lms: Vec<usize> = Self::induced_sort(s, &is_s, sigma, &lms)
            .into_iter()
            .filter(|&i| Self::is_lms(&is_s, i))
            .collect();

        let mut nums = vec![Self::NIL; n];
        let mut num = 0;
        nums[sa_lms[0]] = 0;
        for _i in 1..lms.len() {
            let i = sa_lms[_i - 1];
            let j = sa_lms[_i];
            let mut d = 0;
            loop {
                if s[i + d] != s[j + d] || Self::is_lms(&is_s, i + d) != Self::is_lms(&is_s, j + d)
                {
                    num += 1;
                    break;
                } else if d > 0 && (Self::is_lms(&is_s, i + d) || Self::is_lms(&is_s, j + d)) {
                    break;
                }
                d += 1;
            }
            nums[j] = num;
        }
        let nums: Vec<usize> = nums.into_iter().filter(|&ls| ls != Self::NIL).collect();

        let mut sa_lms = vec![0; num + 1];
        if num + 1 < nums.len() {
            sa_lms = Self::sa_is(&nums, num + 1);
        } else {
            for (i, &x) in nums.iter().enumerate() {
                sa_lms[x] = i;
            }
        }

        let sa_lms: Vec<usize> = sa_lms.iter().map(|&x| lms[x]).collect();

        Self::induced_sort(s, &is_s, sigma, &sa_lms)
    }

    // s, L or S, alphabet size, index of LMS
    fn induced_sort<T>(s: &[T], is_s: &bitvec::Bitvec, sigma: usize, lms: &[usize]) -> Vec<usize>
    where
        T: Ord + Into<usize> + Copy,
    {
        let n = s.len();
        let mut sa = vec![0; n];
        // count alphabets
        let mut b = vec![0; sigma + 1];
        for &c in s.iter() {
            b[c.into() + 1] += 1;
        }
        // 累積和
        for i in 0..sigma {
            b[i + 1] += b[i];
        }

        // step 1: set LMS
        let mut cnt = vec![0; sigma];
        for &i in lms.iter().rev() {
            let c = s[i];
            cnt[c.into()] += 1;
            sa[b[c.into() + 1] - cnt[c.into()]] = i;
        }

        // step 2: set L
        let mut cnt = vec![0; sigma];
        for _i in 0..n {
            let i = sa[_i];
            if i != 0 && !is_s.at(i - 1) {
                let prev = s[i - 1];
                sa[b[prev.into()] + cnt[prev.into()]] = i - 1;
                cnt[prev.into()] += 1;
            }
        }

        // step 3: set S
        let mut cnt = vec![0; sigma];
        for _i in (0..n).rev() {
            let i = sa[_i];
            if i != 0 && is_s.at(i - 1) {
                let prev = s[i - 1];
                cnt[prev.into()] += 1;
                sa[b[prev.into() + 1] - cnt[prev.into()]] = i - 1;
            }
        }

        sa
    }
}
