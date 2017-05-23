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
    fn mutate(generation: &mut[Self]);
    fn procrastinate(parents: &[Self]) -> Self;
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
    // 1. filter strongest
    S::filter_strongest(&mut species);
    
    // 2. bear children
    let mut all_children = vec![];
    for _ in 0..nchildren {
        let families = partition(species, nparents);
        species = vec![];
        let mut children = vec![];
        for family in families.into_iter() {
            if family.len() > 1 {
                //            for _ in 0..nchildren {
                let child = S::procrastinate(&family);
                children.push(child);
                //            }
            }
            species.extend(family)
        }
    
        // 3. mutate
        S::mutate(&mut children);
        for child in children.iter_mut() {
            child.reevaluate();
        }
    
        all_children.extend(children);
    }
    
    if KILL_PARENTS {
        all_children
    } else {
        species.extend(all_children);
        species
    }
}

fn partition<S: Specimen>(mut species: Vec<S>, nparents: usize) -> Vec<Vec<S>> {
    let mut families = vec![];
    let mut family = vec![];
    let mut j = 0;
    let mut rng = RNG.lock().unwrap();
    for _ in 0..species.len() {
        let next = rng.gen_range(0, species.len());
        let s = species.swap_remove(next);
        family.push(s);
    
        j += 1;
        if j == nparents {
            j = 0;
            families.push(family);
            family = vec![];
        }
    }

    if !family.is_empty() {
        families.push(family);
    }

    families
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
//const NCHILDREN: usize = 140;

// BEST FOR SIZE=30
//const KILL_PARENTS: bool = true;
//const SIZE: usize = 30;
//const MUTATION_PROBABILITY: f32 = 1.0f32;
//const POPULATION: usize = 4;
//const MAX_ITERS: usize = 5000;
//const NPARENTS: usize = 2;
//const NCHILDREN: usize = 240;

// BEST FOR SIZE=50
//const KILL_PARENTS: bool = true;
//const SIZE: usize = 50;
//const MUTATION_PROBABILITY: f32 = 1.0f32;
//const POPULATION: usize = 5;
//const MAX_ITERS: usize = 5000;
//const NPARENTS: usize = 2;
//const NCHILDREN: usize = 320;

const KILL_PARENTS: bool = true;
const SIZE: usize = 100;
const MUTATION_PROBABILITY: f32 = 1.0f32;
const POPULATION: usize = 5;
const MAX_ITERS: usize = 5000;
const NPARENTS: usize = 2;
const NCHILDREN: usize = 320;



lazy_static! {
    static ref RNG: Mutex<XorShiftRng> = Mutex::new(XorShiftRng::new_unseeded());
}


struct Board {
    score: f32,
    queens: Vec<(usize, usize)>,
    cells: Vec<bool>
}

impl Board {
    fn new(n: usize, queens: Vec<(usize, usize)>) -> Board {
        let mut cells = vec![false; n*n];
        for &(ref x, ref y) in queens.iter() {
            cells[n*y + x] = true;
        }
        
        Board { score: 0f32, queens: queens, cells: cells }
    }
    
    fn at(&mut self, x: usize, y: usize) -> &mut bool {
        &mut self.cells[SIZE * y + x]
    }
    
    #[inline(never)]
    fn do_mutate(&mut self, rng: &mut XorShiftRng) {
        let q = rng.gen_range(0, SIZE);
        loop {
            let x = rng.gen_range(0, SIZE);
            let y = rng.gen_range(0, SIZE);
            if !*self.at(x, y) {
                let (oldx, oldy) = self.queens[q].clone();
                *self.at(oldx, oldy) = false;
                *self.at(x, y) = true;
                self.queens[q] = (x, y);
                break;
            }
        }
    }
    
    fn eval_conflicts(&self, q1: usize, conflicts: &mut [usize]) {
        let &(ref x1, ref y1) = &self.queens[q1];
        
        for q2 in q1+1..SIZE {
            let &(ref x2, ref y2) = &self.queens[q2];
            if Self::conflict(*x1, *y1, *x2, *y2) {
                conflicts[q1] += 1;
                conflicts[q2] += 1;
            }
        }
    }
    
    fn conflict(x1: usize, y1: usize, x2: usize, y2: usize) -> bool {
        x1 == x2
            || y1 == y2
            || x1.wrapping_sub(x2) == y1.wrapping_sub(y2)
            || x1.wrapping_sub(x2) == y2.wrapping_sub(y1)
    }
}

impl Specimen for Board {
    #[inline(never)]
    fn reevaluate(&mut self) {
        let mut score: usize = 0;
        let mut conflicts = vec![0; SIZE];
        for i in 0 .. SIZE {
            self.eval_conflicts(i, &mut conflicts);
            
            if conflicts[i] == 0 {
                score += 1;
            }
        }
        
        self.score = score as f32 + 0.000001;
    }
    
    fn score(&self) -> f32 {
        self.score
    }
    
    #[inline(never)]
    fn mutate(new_gen: &mut [Self]) {
        let mut rng = RNG.lock().unwrap();
        for board in new_gen.iter_mut() {
            if rng.next_f32() < MUTATION_PROBABILITY {
                board.do_mutate(&mut rng);
            }
        }
    }
    
    #[inline(never)]
    fn procrastinate(parents: &[Self]) -> Self {
        let nqueens = parents.len() * SIZE;
        
        let mut child = Board::new(SIZE, Vec::with_capacity(SIZE));
        
        let mut all_queens = (0..nqueens).collect::<Vec<_>>();
    
        let mut rng = RNG.lock().unwrap();
        loop {
            let next = rng.gen_range(0, all_queens.len());
            let mut queen = all_queens.swap_remove(next);
            let parent = queen / SIZE;
            queen %= SIZE;
    
            let (x, y) = parents[parent].queens[queen].clone();
            if !*child.at(x, y) {
                child.queens.push((x, y));
                *child.at(x, y) = true;
                if child.queens.len() == SIZE {
                    break;
                }
            }
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
            let mut board = Board::new(SIZE, vec![]);
    
            loop {
                let x = rng.gen_range(0, SIZE);
                let y = rng.gen_range(0, SIZE);
                if !*board.at(x, y) {
                    board.queens.push((x, y));
                    *board.at(x, y) = true;
                    if board.queens.len() == SIZE {
                        break;
                    }
                }
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
    genetic_queens()
}

