use std::collections::BinaryHeap;
use std::time::Instant;

mod bitvec;
mod suffix_array;

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

    let clock = Instant::now();
    let rle = zero_run_length(&mtf);
    println!("{:?} for encode RLE", clock.elapsed());

    let clock = Instant::now();
    let huffman_tree = HuffmanTree::new(&rle);
    println!("{:?} for build the Huffman tree", clock.elapsed());

    let clock = Instant::now();
    let encoded = huffman_encode_with_tree(&rle, &huffman_tree);
    println!("{:?} for encode Huffman", clock.elapsed());

    // let huffman_tree = HuffmanTree::new(&s_as_bytes);
    // let encoded = huffman_encode_with_tree(&s_as_bytes, &huffman_tree);
    println!("{} -> {}", s_as_bytes.len(), encoded.len() / 8);
}

#[derive(Debug)]
struct Bwt {
    bwt: Vec<u8>,
}

impl Bwt {
    fn encode(s: &[u8]) -> Bwt {
        let sa = suffix_array::SuffixArray::new(s);
        let n = sa.s.len();
        let mut bwt = vec![0; n];
        for (i, x) in sa.sa.iter().enumerate() {
            bwt[i] = sa.s[(x + n - 1) % n];
        }

        Bwt { bwt: bwt }
    }
}

// mnt2
fn mtf_encode(s: &[u8]) -> Vec<u8> {
    let mut table: Vec<u8> = (0..=255).collect();

    let mut res = vec![0; s.len()];
    let mut prev = 1;
    for (i, x) in s.iter().enumerate() {
        let pos = table.iter().position(|a| a == x).unwrap();
        res[i] = pos;
        for j in (1..pos).rev() {
            table.swap(j, j + 1);
        }
        if pos == 1 && prev != 0 {
            table.swap(0, 1);
        }
        prev = pos;
    }

    res.into_iter().map(|x| x as u8).collect()
}

enum HuffmanTreeRepresentation {
    InternalNode,
    Alphabet(u16),
}

struct HuffmanTree {
    table: Vec<Vec<bool>>,
    tree: Vec<HuffmanTreeRepresentation>,
}

impl HuffmanTree {
    fn new(bytes: &[u16]) -> Self {
        let mut cnt = vec![0; 65536];
        for x in bytes.iter() {
            cnt[*x as usize] += 1;
        }

        let mut heap = BinaryHeap::new();
        for (i, x) in cnt.iter().enumerate().filter(|&(_, x)| *x > 0) {
            // println!("{}, {}", i, x);
            heap.push(HuffmanNode::new(u16::try_from(i).unwrap(), *x));
        }

        while heap.len() > 1 {
            let a = heap.pop().unwrap();
            let b = heap.pop().unwrap();
            heap.push(a.merge(b));
        }

        // Huffman table
        let mut table = vec![Vec::new(); 65536];
        // HuffmanTree Tree の表現
        let mut tree = vec![HuffmanTreeRepresentation::InternalNode];
        heap.pop()
            .unwrap()
            .to_table(&mut Vec::new(), &mut table, &mut tree);

        HuffmanTree { table, tree }
    }
}

#[derive(PartialEq, PartialOrd, Eq, Ord)]
struct HuffmanNode {
    count: isize,
    label: Option<u16>,
    left: Option<Box<HuffmanNode>>,
    right: Option<Box<HuffmanNode>>,
}

impl HuffmanNode {
    fn new(label: u16, count: isize) -> Self {
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

fn huffman_encode_with_tree(bytes: &[u16], huffman_tree: &HuffmanTree) -> Vec<bool> {
    let mut bits = Vec::new();
    for b in bytes.iter() {
        for f in huffman_tree.table[*b as usize].iter() {
            bits.push(*f);
        }
    }
    bits
}

fn zero_run_length(bytes: &[u8]) -> Vec<u16> {
    let mut cnt = 0;
    let mut res: Vec<u16> = Vec::new();
    for &x in bytes.iter() {
        if 0 != x {
            if cnt != 0 {
                res.push(0);
                res.push(cnt - 1);
                cnt = 0;
            }
            res.push(x.try_into().unwrap());
        } else {
            cnt += 1;
        }
    }
    if cnt != 0 {
        res.push(0);
        res.push(cnt - 1);
        // cnt = 0;
    }

    res
}
