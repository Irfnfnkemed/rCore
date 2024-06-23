pub struct LinearCongruentialGenerator {
    a: usize,
    c: usize,
    m: usize,
    x: usize,
}

impl LinearCongruentialGenerator {
    pub(crate) fn new(a: usize, c: usize, m: usize, seed: usize) -> Self {
        LinearCongruentialGenerator { a, c, m, x: seed }
    }

    pub fn next(&mut self) -> usize {
        self.x = (self.a.wrapping_mul(self.x).wrapping_add(self.c)) % self.m;
        self.x
    }
}

fn main() {
    let a = 1664525;
    let c = 1013904223;
    let m = 2usize.pow(32); // 2^32

    // 创建一个种子
    let seed = 123456789;

    // 创建生成器实例
    let mut rng = LinearCongruentialGenerator::new(a, c, m, seed);

    // 生成一些随机数
    for _ in 0..10 {
        println!("Random number: {}", rng.next());
    }
}