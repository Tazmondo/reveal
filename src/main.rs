// Written by Tazm0ndo
// Thanks to the team behind full-moon, and the team behind Rojo!

use std::thread;

const STACK_SIZE: usize = 4 * 1024 * 1024;

fn run() {
    reveal::run("./example");
}

fn run_with_bigger_stack(func: fn() -> ()) {
    // Spawn thread with explicit stack size
    let child = thread::Builder::new()
        .stack_size(STACK_SIZE)
        .spawn(func)
        .unwrap();

    // Wait for thread to join
    child.join().unwrap();
}

fn main() {
    // In debug mode, the full moon parser creates a stack overflow
    // So we run with a larger stack size
    if cfg!(debug_assertions) {
        run_with_bigger_stack(run)
        // let x = full_moon::parse("require(script.Parent.Test)").unwrap();
        // println!("{:?}", x);
    } else {
        run()
    }
}
