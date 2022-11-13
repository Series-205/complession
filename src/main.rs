use std::collections::BinaryHeap;
use std::time::Instant;
// use bitvec::*;

fn main() {
    let mut s_as_bytes = include_bytes!("../sample/bible.txt").to_vec();

    // 0x00 が含まれないことを確認
    for &x in s_as_bytes.iter() {
        assert_ne!(x, 0x00);
    }
    s_as_bytes.push(0x00u8);

    let clock = Instant::now();
    
    let bwted = Bwt::encode(&s_as_bytes);
    println!("{:?} for encode BWT", clock.elapsed());

    let clock = Instant::now();
    
    let mtf = mtf_encode(&bwted.bwt);
    println!("{:?} for encode MTF", clock.elapsed());

    // println!("{:?}\n{:?}", sa.sa, sa.s);
    // println!("{}", String::from_utf8_lossy(&bwted.bwt));
    // println!("{:?}", mtf);

    let clock = Instant::now();
    let huffman_tree = HuffmanTree::new(&mtf);
    println!("{:?} for build the Huffman tree", clock.elapsed());

    let clock = Instant::now();
    let encoded = huffman_encode_with_tree(&mtf, &huffman_tree);
    println!("{:?} for encode Huffman", clock.elapsed());

    // let encoded = gamma_encode(&mtf);

    // let huffman_tree = HuffmanTree::new(&s_as_bytes);
    // let encoded = huffman_encode_with_tree(&s_as_bytes, &huffman_tree);
    println!("{} -> {}", s_as_bytes.len(), encoded.len() / 8);
}

#[derive(Debug)]
struct SuffixArray {
    sa: Vec<usize>,
    s: Vec<u8>,
}

impl SuffixArray {
    fn new(s: &[u8]) -> SuffixArray {
        let n = s.len();
        let mut sa: Vec<usize> = (0..n).collect();
        sa.sort_by(|&a, &b| {
            if s[a] == s[b] {
                b.cmp(&a)
            } else {
                s[a].cmp(&s[b])
            }
        });
        let mut classes = vec![0; n];
        let mut c: Vec<usize> = s.into_iter().map(|&x| x as usize).collect();
        let mut cnt: Vec<usize>;

        let mut len = 1;
        while len < n {
            for i in 0..n {
                classes[sa[i]] = if i > 0
                    && c[sa[i - 1]] == c[sa[i]]
                    && sa[i - 1] + len < n
                    && c[sa[i - 1] + len / 2] == c[sa[i] + len / 2]
                {
                    classes[sa[i - 1]]
                } else {
                    i
                };
            }

            cnt = (0..n).collect();
            c = sa.clone();

            for i in 0..n {
                if c[i] >= len {
                    let s1 = c[i] - len;
                    sa[cnt[classes[s1]]] = s1;
                    cnt[classes[s1]] += 1;
                }
            }

            c = classes.clone();

            len *= 2;
        }

        SuffixArray {
            sa: sa,
            s: s.to_vec(),
        }
    }
}

#[derive(Debug)]
struct Bwt {
    bwt: Vec<u8>,
}

impl Bwt {
    fn encode(s: &[u8]) -> Bwt {
        let sa = SuffixArray::new(s);
        let n = sa.s.len();
        let mut bwt = vec![0; n];
        for (i, x) in sa.sa.iter().enumerate() {
            bwt[i] = sa.s[(x + n - 1) % n];
        }

        Bwt { bwt: bwt }
    }
}

fn mtf_encode(s: &[u8]) -> Vec<u8> {
    let mut table: Vec<u8> = (0..=255).collect();

    let mut res = vec![0; s.len()];
    for (i, x) in s.iter().enumerate() {
        let pos = table.iter().position(|a| a == x).unwrap();
        res[i] = pos;
        for j in (0..pos).rev() {
            table.swap(j, j + 1);
        }
    }

    res.into_iter().map(|x| x as u8).collect()
}

enum HuffmanTreeRepresentation {
    InternalNode,
    Alphabet(u8),
}

struct HuffmanTree {
    table: Vec<Vec<bool>>,
    tree: Vec<HuffmanTreeRepresentation>,
}

impl HuffmanTree {
    fn new(bytes: &[u8]) -> Self {
        let mut cnt = vec![0; 256];
        for x in bytes.iter() {
            cnt[*x as usize] += 1;
        }

        let mut heap = BinaryHeap::new();
        for (i, x) in cnt.iter().enumerate().filter(|&(_, x)| *x > 0) {
            // println!("{}, {}", i, x);
            heap.push(HuffmanNode::new(u8::try_from(i).unwrap(), *x));
        }

        while heap.len() > 1 {
            let a = heap.pop().unwrap();
            let b = heap.pop().unwrap();
            heap.push(a.merge(b));
        }

        // Huhhman table
        let mut table = vec![Vec::new(); 256];
        // HuffmanTree Tree の括弧列表現
        let mut tree = vec![HuffmanTreeRepresentation::InternalNode];
        heap.pop()
            .unwrap()
            .to_table(&mut Vec::new(), &mut table, &mut tree);

        for (i, x) in cnt.iter().enumerate().filter(|&(_, x)| *x > 0) {
            // println!("{}, {}, {}", i, x, table[i].len());
        }

        HuffmanTree {
            table: table,
            tree: tree,
        }
    }
}

#[derive(PartialEq, PartialOrd, Eq, Ord)]
struct HuffmanNode {
    count: isize,
    label: Option<u8>,
    left: Option<Box<HuffmanNode>>,
    right: Option<Box<HuffmanNode>>,
}

impl HuffmanNode {
    fn new(label: u8, count: isize) -> Self {
        HuffmanNode {
            count: -count,
            label: Some(label),
            left: None,
            right: None,
        }
    }
    fn merge(self, other: Self) -> Self {
        let count = self.count + other.count;
        HuffmanNode {
            count: count,
            label: None,
            left: Some(Box::new(self)),
            right: Some(Box::new(other)),
        }
    }
    fn to_table(
        self,
        bits: &mut Vec<bool>,
        table: &mut [Vec<bool>],
        tree: &mut Vec<HuffmanTreeRepresentation>,
    ) {
        match self.label {
            Some(label) => {
                tree.push(HuffmanTreeRepresentation::Alphabet(label));
                table[label as usize] = bits.clone();
            }
            None => {
                tree.push(HuffmanTreeRepresentation::InternalNode);
                bits.push(false);
                self.left.unwrap().to_table(bits, table, tree);
                bits.pop();
                bits.push(true);
                self.right.unwrap().to_table(bits, table, tree);
                bits.pop();
            }
        }
    }
}

// tree -> code の順番
fn huffman_encode_with_tree(bytes: &[u8], huffman_tree: &HuffmanTree) -> Vec<bool> {
    let mut bits = Vec::new();
    for b in bytes.iter() {
        for f in huffman_tree.table[*b as usize].iter() {
            bits.push(*f);
        }
    }
    bits
}

// byte: positive number
fn gamma(byte: u8) -> Vec<bool> {
    let len = 8 - byte.leading_zeros();
    let mut res = vec![false; len as usize - 1];
    for i in (0..len).rev() {
        res.push(byte >> i & 1 == 1);
    }
    res
}

fn gamma_encode(bytes: &[u8]) -> Vec<bool> {
    let mut res = Vec::new();
    for x in bytes.iter() {
        // 0 は符号化できないので 1 足す
        for b in gamma(*x + 1) {
            res.push(b);
        }
    }
    res
}