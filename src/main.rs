extern crate rand;

#[macro_use]
extern crate lazy_static;


use rand::{XorShiftRng, Rng, thread_rng, SeedableRng};
use std::fmt;
use std::fmt::{Formatter, Debug};

use std::sync::Mutex;

trait Specimen: Sized {
    fn score(&self) -> f32;
    fn reevaluate(&mut self);
    fn mutate(generation: &mut[Self], rng: &mut XorShiftRng);
    fn procrastinate(parents: &[Self], rng: &mut XorShiftRng) -> Self;
    fn filter_strongest(species: &mut Vec<Self>);
    
    fn initial() -> Vec<Self>;
}


fn genetic<S: Specimen>(threshold: f32, max_iters: usize, nparents: usize, nchildren: usize) -> S {
    let mut species: Vec<S> = S::initial();
    let mut best = (0f32, 0);
    for _ in 0..max_iters {
        species = next_gen(species, nparents, nchildren);
        
        for (i, specimen) in species.iter().enumerate() {
            let score = specimen.score();
            if best.0 <= score {
                best = (score, i)
            }
        }
        
        if best.0 >= threshold {
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
        let child = S::procrastinate(&family, &mut rng);
        children.push(child);
        species.extend(family.drain(..));
    }
    
    // 3. mutate
    S::mutate(&mut children, &mut rng);
    for child in children.iter_mut() {
        child.reevaluate();
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

// BEST FOR SIZE=8
//const KILL_PARENTS: bool = true;
//const SIZE: usize = 8;
//const MUTATION_PROBABILITY: f32 = 1.0f32;
//const POPULATION: usize = 3;
//const MAX_ITERS: usize = 1000;
//const NPARENTS: usize = 2;
//const NCHILDREN: usize = 100;

// BEST FOR SIZE=12
//const KILL_PARENTS: bool = true;
//const SIZE: usize = 12;
//const MUTATION_PROBABILITY: f32 = 1.0f32;
//const POPULATION: usize = 3;
//const MAX_ITERS: usize = 1000;
//const NPARENTS: usize = 2;
//const NCHILDREN: usize = 100;

// BEST FOR SIZE=15
//const KILL_PARENTS: bool = true;
//const SIZE: usize = 15;
//const MUTATION_PROBABILITY: f32 = 1.0f32;
//const POPULATION: usize = 4;
//const MAX_ITERS: usize = 1000;
//const NPARENTS: usize = 2;
//const NCHILDREN: usize = 280;

// BEST FOR SIZE=30
//const KILL_PARENTS: bool = true;
//const SIZE: usize = 30;
//const MUTATION_PROBABILITY: f32 = 1.0f32;
//const POPULATION: usize = 4;
//const MAX_ITERS: usize = 5000;
//const NPARENTS: usize = 2;
//const NCHILDREN: usize = 480;

// BEST FOR SIZE=50
const KILL_PARENTS: bool = true;
const SIZE: usize = 400;
const MUTATION_PROBABILITY: f32 = 1.0f32;
const POPULATION: usize = 5;
const MAX_ITERS: usize = 5000;
const NPARENTS: usize = 2;
const NCHILDREN: usize = 640;

//const KILL_PARENTS: bool = true;
//const SIZE: usize = 100;
//const MUTATION_PROBABILITY: f32 = 1.0f32;
//const POPULATION: usize = 5;
//const MAX_ITERS: usize = 5000;
//const NPARENTS: usize = 2;
//const NCHILDREN: usize = 1200;



lazy_static! {
    static ref RNG: Mutex<XorShiftRng> = Mutex::new(XorShiftRng::new_unseeded());
}


struct Board {
    score: f32,
    queens: Vec<usize>,
}

impl Board {
    fn new(queens: Vec<usize>) -> Board {
        Board { score: 0f32, queens: queens }
    }
    
    #[inline(never)]
    fn do_mutate(&mut self, rng: &mut XorShiftRng) {
        let x = rng.gen_range(0, SIZE);
        
        let mut y = rng.gen_range(0, SIZE-1);
        if y >= self.queens[x] {
            y += 1;
        }
        self.queens[x] = y;
    }
}

impl Specimen for Board {
    #[inline(never)]
    fn reevaluate(&mut self) {
        let mut occurences = vec![0; 2*SIZE-1];
        let mut nonconflicting = Vec::with_capacity(SIZE);
        
        // columns
        for x in 0..SIZE {
            occurences[self.queens[x]] += 1;
        }
        for x in 0..SIZE {
            if occurences[self.queens[x]] == 1 {
                nonconflicting.push(x);
            }
        }
        
        // forward diagonals
        for y in 0..SIZE {
            occurences[y] = 0;
        }
        for x in 0..SIZE {
            let d = SIZE-1 + x - self.queens[x];
            occurences[d] += 1;
        }
        let mut i = 0;
        while i < nonconflicting.len() {
            let x = nonconflicting[i];
            let d = SIZE-1 + x - self.queens[x];
            if occurences[d] == 1 {
                i += 1;
            } else {
                nonconflicting.swap_remove(i);
            }
        }
    
        // backward diagonals
        for y in 0..2*SIZE-1 {
            occurences[y] = 0;
        }
        for x in 0..SIZE {
            let d = x + self.queens[x];
            occurences[d] += 1;
        }
        i = 0;
        while i < nonconflicting.len() {
            let x = nonconflicting[i];
            let d = x + self.queens[x];
            if occurences[d] == 1 {
                i += 1;
            } else {
                nonconflicting.swap_remove(i);
            }
        }
    
        self.score = nonconflicting.len() as f32 + 0.000001;
    }
    
    fn score(&self) -> f32 {
        self.score
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
    fn procrastinate(parents: &[Self], rng: &mut XorShiftRng) -> Self {
        let mut child = Board::new(Vec::with_capacity(SIZE));
        
        for x in 0..SIZE {
            let parent = rng.gen_range(0, parents.len());
            child.queens.push(parents[parent].queens[x]);
        }
        
        child
    }
    
    #[inline(never)]
    fn filter_strongest(species: &mut Vec<Self>) {
        if species.len() <= POPULATION {
            return;
        }
        
        species.sort_by(|s, t| t.score.partial_cmp(&s.score).unwrap());
        species.truncate(POPULATION);
    }
    
    #[inline(never)]
    fn initial() -> Vec<Self> {
        let mut boards = Vec::with_capacity(POPULATION);
    
        let mut rng = RNG.lock().unwrap();
        for _ in 0..POPULATION {
            let mut board = Board::new(Vec::with_capacity(SIZE));
    
            for _ in 0..SIZE {
                let y = rng.gen_range(0, SIZE);
                board.queens.push(y);
            }
    
            board.reevaluate();
            boards.push(board);
        }
        boards
    }
}

impl Debug for Board {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "score={}, queens=[{:?}]", self.score, self.queens)
    }
}


fn genetic_queens() {
    {
        let mut rng = RNG.lock().unwrap();
        let mut trng = thread_rng();
        rng.reseed([trng.next_u32(), trng.next_u32(), trng.next_u32(), trng.next_u32()]);
    }
    
    let best = genetic::<Board>(SIZE as f32, MAX_ITERS, NPARENTS, NCHILDREN);
    println!("best: {:?}", &best);
}


fn main() {
    for _ in 0..10 {
        genetic_queens()
    }
}

