extern crate rand;

#[macro_use]
extern crate lazy_static;


use rand::{XorShiftRng, Rng, thread_rng, SeedableRng};
use std::fmt;
use std::fmt::{Formatter, Debug};
use std::sync::Mutex;
use std::cmp;

trait Fitness: Ord {
    fn max() -> Self;
}

trait Specimen: Sized {
    type F: Fitness;
    
    fn fitness(&self) -> Self::F;
    fn reevaluate(&mut self, rng: &mut XorShiftRng);
    fn mutate(generation: &mut[Self], rng: &mut XorShiftRng);
    fn breed(parents: &[Self], rng: &mut XorShiftRng) -> Self;
    fn filter_strongest(species: &mut Vec<Self>);
    
    fn initial() -> Vec<Self>;
}


fn genetic<S: Specimen>(threshold: S::F, max_iters: usize, nparents: usize, nchildren: usize) -> S {
    let mut species: Vec<S> = S::initial();
    let mut best = (S::F::max(), 0);
    for _ in 0..max_iters {
        species = next_gen(species, nparents, nchildren);
        
        for (i, specimen) in species.iter().enumerate() {
            let fitness = specimen.fitness();
            if fitness <= best.0 {
                best = (fitness, i)
            }
        }
        
        if best.0 <= threshold {
            println!("found!");
            break;
        }
    }
    
    species.swap_remove(best.1)
}

#[inline(never)]
fn next_gen<S: Specimen>(mut species: Vec<S>, nparents: usize, nchildren: usize) -> Vec<S> {
    let mut rng = RNG.lock().unwrap();
    
    // 1. filter strongest
    S::filter_strongest(&mut species);
    
    // 2. bear children
    let mut children = vec![];
    let mut family = vec![];
    for _ in 0..nchildren {
        make_family(&mut species, nparents, &mut family, &mut rng);
        let child = S::breed(&family, &mut rng);
        children.push(child);
        species.extend(family.drain(..));
    }
    
    // 3. mutate
    S::mutate(&mut children, &mut rng);
    for child in children.iter_mut() {
        child.reevaluate(&mut rng);
    }
    
    
    if KILL_PARENTS {
        children
    } else {
        species.extend(children);
        species
    }
}

fn make_family<S: Specimen>(species: &mut Vec<S>, nparents: usize, family: &mut Vec<S>, rng: &mut XorShiftRng) {
    assert!(family.is_empty());
    for _ in 0..nparents {
        let next = rng.gen_range(0, species.len());
        let s = species.swap_remove(next);
        family.push(s);
    }
}


//---------------------------------------------------

// BEST FOR SIZE=1000
//const ELITE: usize = 3;
//const NCHILDREN: usize = 65;
//const KILL_PARENTS: bool = false;
//const SIZE: usize = 1000;
//const MUTATION_PROBABILITY: f32 = 1.0f32;
//const MAX_ITERS: usize = 5000;
//const NPARENTS: usize = 2;


//const ELITE: usize = 4;
//const NCHILDREN: usize = 130;
//const KILL_PARENTS: bool = true;
const ELITE: usize = 2;
//const NCHILDREN: usize = 65;
const NCHILDREN: usize = 1;
const KILL_PARENTS: bool = false;
const SIZE: usize = 100000;
const MUTATION_PROBABILITY: f32 = 0.01f32;
const MAX_ITERS: usize = 50000;
const NPARENTS: usize = 2;


//
//const KILL_PARENTS: bool = true;
//const SIZE: usize = 10000;
//const MUTATION_PROBABILITY: f32 = 1.0f32;
//const POPULATION: usize = 20;
//const MAX_ITERS: usize = 5000;
//const NPARENTS: usize = 2;
//const NCHILDREN: usize = 1000;



lazy_static! {
    static ref RNG: Mutex<XorShiftRng> = Mutex::new(XorShiftRng::new_unseeded());
}


struct Board {
    fitness: usize,
    queens: Vec<usize>,
}

impl Board {
    fn new(queens: Vec<usize>) -> Board {
        Board { fitness: ::std::usize::MAX, queens: queens }
    }
    
    #[inline(never)]
    fn do_mutate(&mut self, rng: &mut XorShiftRng) {
        let i = rng.gen_range(0, SIZE);
        let mut j = rng.gen_range(0, SIZE-1);
        
        if j >= i {
            j += 1;
        }
        
        self.queens.swap(i, j);
    }
    
    
    fn breed_pmx_norng(parents: &[Self], rng: &mut XorShiftRng) -> Self {
        let mut child = Board::new(parents[0].queens.clone());
    
        // A very important assert! Without it, the program runs twice slower! o_O
        assert!(parents.len() == NPARENTS);
    
        {
            let cq: &mut [usize] = &mut child.queens;
            let mut inverse = vec![0; SIZE];
            for i in 0..SIZE {
                let y = cq[i];
                inverse[y] = i;
            }
        
            let mut x = rng.gen_range(0, SIZE);
            for _ in 0..SIZE/2 {
                x += 1;
                if x == SIZE {
                    x = 0;
                }
                
                let parent = 1;
                let mother_y = cq[x];
                let father_y = parents[parent].queens[x];
                let father_x = inverse[father_y];
                cq[father_x] = mother_y;
                cq[x] = father_y;
                inverse[mother_y] = father_x;
                if NPARENTS > 2 {
                    // Optimization: this is not required when NPARENTS==2
                    inverse[father_y] = x;
                }
            }
        }

        child
    }
    
    // The algorithm is described in "Genetic Algorithm Solution of the TSP Avoiding Special Crossover and Mutation"
    // http://user.ceng.metu.edu.tr/~ucoluk/research/publications/tspnew.pdf
    #[inline(never)]
    fn breed_pmx(parents: &[Self], rng: &mut XorShiftRng) -> Self {
        // A very important assert! Without it, the program runs twice slower! o_O
        assert!(parents.len() == NPARENTS);
        
        let mut child = Board::new(parents[0].queens.clone());
        {
            let mut inverse = vec![0; SIZE];
            let cq = &mut child.queens;
            for i in 0..SIZE {
                let y = cq[i];
                inverse[y] = i;
            }
        
            for x in 0..SIZE {
                let parent = rng.gen_range(0, parents.len());
                if parent == 1 {
                    let mother_y = cq[x];
                    let father_y = parents[parent].queens[x];
                    let father_x = inverse[father_y];
                    cq[father_x] = mother_y;
                    cq[x] = father_y;
                    inverse[mother_y] = father_x;
                    if NPARENTS > 2 { // Optimization: this is not required when NPARENTS==2
                        inverse[father_y] = x;
                    }
                }
            }
        }
        
        child
    }
    
//    fn permutation_valid(p: &[usize]) -> bool {
//        let mut used = vec![0; p.len()];
//        for &i in p.iter() {
//            used[i] = 1;
//        }
//        for &u in used.iter() {
//            if u != 1 {
//                return false;
//            }
//        }
//
//        true
//    }
}


impl Fitness for usize {
    fn max() -> Self {
        ::std::usize::MAX
    }
}

impl Specimen for Board {
    type F = usize;
    
    #[inline(never)]
    fn reevaluate(&mut self, rng: &mut XorShiftRng) {
        let q: &mut [usize] = &mut self.queens;
        let mut diag_counts = vec![0; 4*SIZE-2];
        let (diag_counts1, diag_counts2) = diag_counts.split_at_mut(2*SIZE-1);
        for x in 0..SIZE {
            let d1 = SIZE-1 + x - q[x];
            let d2 = x + q[x];
            diag_counts1[d1] += 1;
            diag_counts2[d2] += 1;
        }

        let mut f = 0;
        for x in 0..SIZE {
            let d1 = SIZE-1 + x - q[x];
            let d2 = x + q[x];
            f += diag_counts1[d1] + diag_counts2[d2];
        }
    
        self.fitness = f;
    
    
        // perform a conflict minimization step
        let i = rng.gen_range(0, SIZE);
        let qi = q[i];
        let (di1o, di2o) = (SIZE-1 + i - qi, i + qi);
        let (ci1o, ci2o) = (diag_counts1[di1o], diag_counts2[di2o]);
        let ci_old = ci1o + ci2o;
        let mut max_deltaf = 0;
        let mut max_deltaf_j = 0;
        for j in 0..SIZE {
            if i == j {
                continue;
            }
    
            let qj = q[j];
            
            let (dj1o, dj2o) = (SIZE-1 + j - qj, j + qj);
            let (cj1o, cj2o) = (diag_counts1[dj1o], diag_counts2[dj2o]);
            let cj_old = cj1o + cj2o;
            
            if ci_old==2 && cj_old==2 {
                continue;
            }
            
            let (di1n, di2n) = (SIZE-1 + j - qi, j + qi);
            let (ci1n, ci2n) = (diag_counts1[di1n], diag_counts2[di2n]);
            
            let (dj1n, dj2n) = (SIZE-1 + i - qj, i + qj);
            let (cj1n, cj2n) = (diag_counts1[dj1n], diag_counts2[dj2n]);
            
            let fold;
            let fnew;
            if di1o == dj1o {
                assert!(di2n == dj2n);
                let c1o = (ci1o-1)*4; // neg
                let c2n = (ci2n+1)*4;
    
                let c2o = ci2o + cj2o;
                let c1n = ci1n + cj1n;
                
                fold = c1o + (c2o - 1)*2;
                fnew = c2n + (c1n + 1)*2;
            } else if di2o == dj2o {
                assert!(di1n == dj1n);
                let c2o = (ci2o-1)*4; // neg
                let c1n = (ci1n+1)*4;
    
                let c1o = ci1o + cj1o; // neg
                let c2n = ci2n + cj2n;
                
                fold = c2o + (c1o - 1)*2;
                fnew = c1n + (c2n + 1)*2;
            } else {
                let ci_new = ci1n + ci2n + 2;
                let cj_new = cj1n + cj2n + 2;
    
                fold = (ci_old + cj_old)*2;
                fnew = (ci_new + cj_new)*2;
            }
    
            if fold > max_deltaf + fnew {
                max_deltaf = fold - fnew;
                max_deltaf_j = j;
            }
        }
        
        if max_deltaf != 0 {
//            println!("queens_o={:?}, fitness_o={}, case={}", q, self.fitness, case);
            q.swap(i, max_deltaf_j);
            self.fitness -= max_deltaf;
            
//            // TEST DEBUG
//            let mut diag_counts = vec![0; 4*SIZE-2];
//            let (diag_counts1, diag_counts2) = diag_counts.split_at_mut(2*SIZE-1);
//            for x in 0..SIZE {
//                let d1 = SIZE-1 + x - q[x];
//                let d2 = x + q[x];
//                diag_counts1[d1] += 1;
//                diag_counts2[d2] += 1;
//            }
//
//            let mut f = 0;
//            for x in 0..SIZE {
//                let d1 = SIZE-1 + x - q[x];
//                let d2 = x + q[x];
//                f += diag_counts1[d1] + diag_counts2[d2];
//            }
//
//            assert!(f == self.fitness, "max_deltaf={}, f={}, fitness={}, queens={:?}", max_deltaf, f, self.fitness, q);
        }
    }
    
    fn fitness(&self) -> usize {
        self.fitness
    }
    
    #[inline(never)]
    fn mutate(new_gen: &mut [Self], rng: &mut XorShiftRng) {
        for board in new_gen.iter_mut() {
            if rng.next_f32() < MUTATION_PROBABILITY {
                board.do_mutate(rng);
            }
        }
    }
    
    #[inline(never)]
    fn breed(parents: &[Self], rng: &mut XorShiftRng) -> Self {
//        Self::breed_pmx(parents, rng)
        Self::breed_pmx_norng(parents, rng)
    }
    
    #[inline(never)]
    fn filter_strongest(species: &mut Vec<Self>) {
        if species.len() <= ELITE {
            return;
        }
        
        species.sort_by(|s, t| s.fitness.partial_cmp(&t.fitness).unwrap());
        species.truncate(ELITE);
    }
    
    #[inline(never)]
    fn initial() -> Vec<Self> {
        let population = cmp::max(NCHILDREN, NPARENTS);
        let mut boards = Vec::with_capacity(population);
        
        let mut ys = Vec::with_capacity(SIZE);
    
        let mut rng = RNG.lock().unwrap();
        for _ in 0..population {
            let mut board = Board::new(Vec::with_capacity(SIZE));
            
            for y in 0..SIZE {
                ys.push(y);
            }
    
            for _ in 0..SIZE {
                let z = rng.gen_range(0, ys.len());
                let y = ys.swap_remove(z);
                board.queens.push(y);
            }
    
            board.reevaluate(&mut rng);
            boards.push(board);
        }
        boards
    }
}

impl Debug for Board {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "fitness={}, queens=[{:?}]", self.fitness, self.queens)
    }
}


fn genetic_queens() {
    {
        let mut rng = RNG.lock().unwrap();
        let mut trng = thread_rng();
        rng.reseed([trng.next_u32(), trng.next_u32(), trng.next_u32(), trng.next_u32()]);
    }
    
    let best = genetic::<Board>(2*SIZE, MAX_ITERS, NPARENTS, NCHILDREN);
//    println!("best: {:?}", &best);
    println!("best: {:?}", best.fitness());
}


fn main() {
    for _ in 0..10 {
        genetic_queens()
    }
}

